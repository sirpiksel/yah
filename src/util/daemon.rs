use std::collections::HashMap;
use std::fs::{self, create_dir_all, File, set_permissions};
use std::io;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process;

use chrono::{format::DelayedFormat, format::StrftimeItems, NaiveTime};
use chrono::Local;

use crate::env;

pub fn cronux(options: &HashMap<String, String>) {
  // Fork the current process to create a child process
  match unsafe { libc::fork() } {
    -1 => {
      eprintln!("Fork failed.");
      process::exit(1);
    }
    0 => unsafe {
      // Setup daemon
      daemonize();
      create_cronux_pipe().expect("Failed to create pipe");

      // Perform tasks here
      loop {
        write_cronux_pipe(options).expect("Failed to write to pipe");

        std::thread::sleep(std::time::Duration::from_secs(60));
      }
    }
    _ => {
      // Parent process: Exit
      process::exit(0);
    }
  }
}

unsafe fn daemonize() {
  // Get XDG_CONFIG_HOME environment variable
  let xdg_config_home = match env::var("XDG_CONFIG_HOME") {
    Ok(path) => path,
    Err(_) => {
      eprintln!("XDG_CONFIG_HOME is not set.");
      process::exit(1);
    }
  };

  // Create the directory if it doesn't exist
  let yah_dir = format!("{}/yah", xdg_config_home);
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

fn create_cronux_pipe() -> io::Result<()> {
  let pipe_path = env::current_dir()?.join("cronux");

  // Check if the named pipe already exists.
  if pipe_path.exists() {
    return Ok(());
  }

  // Create the named pipe with the FileType::Fifo type.
  create_dir_all(&pipe_path.parent().unwrap())?; // Ensure parent directory exists.
  File::create(&pipe_path)?;
  set_permissions(&pipe_path, fs::Permissions::from_mode(0o666))?; // Set permissions as needed.

  Ok(())
}

fn write_cronux_pipe(timetable: &HashMap<String, String>) -> io::Result<()> {
  // Get the current time
  let current_time = Local::now().time();
  let current_time_formatted: DelayedFormat<StrftimeItems> = current_time.format("%H:%M");

  // Determine the current activity
  let mut current_activity = "Unknown";
  let mut current_activity_time = NaiveTime::from_hms_opt(0, 0, 0).expect("Failed to create NaiveTime");

  for (activity, start_time_str) in timetable.iter() {
    if let Some(start_time) = NaiveTime::parse_from_str(start_time_str, "%H:%M").ok() {
      if current_time >= start_time && start_time > current_activity_time {
        current_activity = activity;
        current_activity_time = start_time;
      }
    }
  }

  // Open the named pipe "cronux" in write-only mode
  let pipe_path = Path::new("./cronux");
  let mut pipe = File::create(pipe_path)?;

  // Write the current activity and starting time to the pipe
  let entry = format!("{} {}\n", current_activity, current_time_formatted);
  pipe.write_all(entry.as_bytes())?;

  Ok(())
}
