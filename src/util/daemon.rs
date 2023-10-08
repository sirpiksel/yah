use std::collections::HashMap;
use std::env;
use std::fs::{self, create_dir_all, File, set_permissions};
use std::io;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process;
use std::thread::sleep;
use std::time::{Duration, Instant};

use alto::{Alto, Mono, Source, Stereo};
use chrono::Local;
use chrono::NaiveTime;
use lewton::inside_ogg::OggStreamReader;
use lewton::VorbisError;

unsafe fn daemonize() {
  // Get XDG_CONFIG_HOME environment variable
  let env_config = match env::var("XDG_CONFIG_HOME") {
    Ok(path) => path,
    Err(_) => {
      eprintln!("XDG_CONFIG_HOME is not set.");
      process::exit(1);
    }
  };

  // Create the directory if it doesn't exist
  let yah_dir = format!("{}/yah", env_config);
  if let Err(err) = create_dir_all(&yah_dir) {
    eprintln!("Failed to create directory: {}", err);
    process::exit(1);
  }

  // Change the working directory to yah_dir
  if let Err(err) = env::set_current_dir(&yah_dir) {
    eprintln!("Failed to change working directory: {}", err);
    process::exit(1);
  }

  // Redirect standard file descriptors to /dev/null
  let dev_null = File::open("/dev/null").expect("Failed to open /dev/null");
  if let Err(err) = dup2(dev_null.as_raw_fd(), 0) {
    eprintln!("Failed to redirect stdin: {}", err);
    process::exit(1);
  }
  if let Err(err) = dup2(dev_null.as_raw_fd(), 1) {
    eprintln!("Failed to redirect stdout: {}", err);
    process::exit(1);
  }
  if let Err(err) = dup2(dev_null.as_raw_fd(), 2) {
    eprintln!("Failed to redirect stderr: {}", err);
    process::exit(1);
  }
}

fn dup2(src: std::os::unix::io::RawFd, dst: std::os::unix::io::RawFd) -> io::Result<()> {
  // Duplicate the file descriptor using the `fcntl` system call
  if unsafe { libc::fcntl(src, libc::F_DUPFD_CLOEXEC, dst) } != -1 {
    Ok(())
  } else {
    Err(io::Error::last_os_error())
  }
}

fn create_cronux_config() -> io::Result<()> {
  let pipe_path = env::current_dir()?.join("cronux");
  let sound_path = env::current_dir()?.join("sound.ogg");

  // Get the sound file
  let sound_bytes = include_bytes!("../../sound.ogg");

  // Check if the named pipe already exists.
  if !pipe_path.exists() {
    // Create the named pipe with the FileType::Fifo type.
    File::create(&pipe_path)?;
    set_permissions(&pipe_path, fs::Permissions::from_mode(0o644))?;
  }

  // Check if sound file already exists.
  if !sound_path.exists() {
    // Create the sound file
    fs::write(&sound_path, sound_bytes)?;
    set_permissions(&sound_path, fs::Permissions::from_mode(0o644))?;
  }

  Ok(())
}

fn write_cronux_pipe(timetable: &HashMap<String, String>) -> io::Result<()> {
  // Get the current time
  let current_time = Local::now().time();

  let mut activity_time = NaiveTime::from_hms_opt(0, 0, 0).expect("Failed to create NaiveTime");
  let mut next_activity_time = NaiveTime::from_hms_opt(23, 59, 59).expect("Failed to create NaiveTime");
  let mut activity_name = "Unknown";

  // Determine the current activity
  for (result_time_str, result_name) in timetable.iter() {
    if let Some(result_time) = NaiveTime::parse_from_str(result_time_str, "%R").ok() {
      if result_time <= current_time && activity_time <= result_time {
        activity_name = result_name;
        activity_time = result_time;
      }
      if current_time < result_time && result_time < next_activity_time {
        next_activity_time = result_time;
      }
    }
  }

  // play sound if the activity just changed
  if current_time == activity_time {
    match play_sound() {
      Ok(_) => (),
      Err(err) => eprintln!("Failed to play sound: {}", err),
    }
  }

  // Open the named pipe "cronux" in write-only mode
  let pipe_path = Path::new("./cronux");
  let mut pipe = File::create(pipe_path)?;

  // Calculate time done & time remaining
  let time_done = (current_time - activity_time).num_minutes().to_string();
  let time_remaining = (next_activity_time - current_time).num_minutes().to_string();


  // Write the current activity and starting time to the pipe
  let entry = format!("{} +{} -{}", activity_name, time_done, time_remaining);
  pipe.write_all(entry.as_bytes())?;

  Ok(())
}

fn play_sound() -> Result<(), VorbisError> {
  let f = File::open("sound.ogg").expect("Failed to open sound file");

  // Prepare the reading
  let mut srr = OggStreamReader::new(f)?;

  // Prepare the playback.
  let al = Alto::load_default().expect("Could not load alto");
  let device = al.open(None).expect("Could not open device");
  let cxt = device.new_context(None).expect("Could not create context");
  let mut str_src = cxt.new_streaming_source()
    .expect("could not create streaming src");
  let sample_rate = srr.ident_hdr.audio_sample_rate as i32;

  if srr.ident_hdr.audio_channels > 2 {
    // the openal crate can't process these many channels directly
    println!("Stream error: {} channels are too many!", srr.ident_hdr.audio_channels);
  }

  // Decode
  let mut n = 0;
  let mut len_play = 0.0;
  let mut start_play_time = None;
  let start_decode_time = Instant::now();
  let sample_channels = srr.ident_hdr.audio_channels as f32 *
    srr.ident_hdr.audio_sample_rate as f32;
  while let Some(pck_samples) = srr.read_dec_packet_itl()? {
    n += 1;
    let buf = match srr.ident_hdr.audio_channels {
      1 => cxt.new_buffer::<Mono<i16>, _>(&pck_samples, sample_rate),
      2 => cxt.new_buffer::<Stereo<i16>, _>(&pck_samples, sample_rate),
      n => panic!("unsupported number of channels: {}", n),
    }.unwrap();

    str_src.queue_buffer(buf).unwrap();

    // Play
    len_play += pck_samples.len() as f32 / sample_channels;
    // If the decode is faster than realtime start playing now.
    if n == 100 {
      let cur = Instant::now();
      if cur - start_decode_time < Duration::from_millis((len_play * 1000.0) as u64) {
        start_play_time = Some(cur);
        str_src.play();
      }
    }
  }
  let total_duration = Duration::from_millis((len_play * 1000.0) as u64);
  let sleep_duration = total_duration - match start_play_time {
    None => {
      str_src.play();
      Duration::from_millis(0)
    }
    Some(t) => Instant::now() - t,
  };
  sleep(sleep_duration);

  Ok(())
}

pub fn daemon(options: &HashMap<String, String>) {
  // Fork the current process to create a child process
  match unsafe { libc::fork() } {
    -1 => {
      eprintln!("Fork failed.");
      process::exit(1);
    }
    0 => unsafe {
      // Setup daemon
      daemonize();
      create_cronux_config().expect("Failed to create pipe");

      // Perform tasks here
      loop {
        write_cronux_pipe(options).expect("Failed to write to pipe");

        sleep(Duration::from_secs(60));
      }
    }
    _ => {
      // Parent process: Exit
      process::exit(0);
    }
  }
}

pub fn cronux() {
  let env_config = match env::var("XDG_CONFIG_HOME") {
    Ok(path) => path,
    Err(_) => {
      eprintln!("XDG_CONFIG_HOME is not set.");
      process::exit(1);
    }
  };
  let pipe = format!("{}/yah/cronux", env_config).to_string();

  // Attempt to open the named pipe for reading.
  match File::open(pipe.clone()) {
    Ok(file) => {
      // Create a buffer to read the data into.
      let mut buffer = Vec::new();

      // Read from the named pipe and store it in the buffer.
      if let Err(err) = file.take(1024).read_to_end(&mut buffer) {
        eprintln!("Error reading from named pipe: {:?}", err);
        return;
      }

      // Convert the buffer to a string and print it.
      if let Ok(contents) = String::from_utf8(buffer) {
        println!("{}", contents);
      } else {
        eprintln!("Failed to convert pipe contents to a string.");
      }
    }
    Err(err) => {
      eprintln!("Error opening named pipe: {:?}", err);
    }
  }
}
