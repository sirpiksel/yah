use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn dmenu(map: &HashMap<String, String>) -> Option<String> {
  // setup dmenu command
  let mut selector = Command::new("dmenu")
    .args(["-c", "-l", "10"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to run dmenu.");

  // pipe in keys
  let mut keys: Vec<&String> = map.keys().collect();
  keys.sort();
  let keys_str: String = keys.iter().map(|key| key.as_str()).collect::<Vec<&str>>().join("\n");
  let mut opt_in = selector.stdin.take().expect("Failed to open stdin.");
  std::thread::spawn(move || {
    opt_in.write_all(keys_str.as_bytes()).expect("Failed to write to stdin.");
  });

  // extract sanitized value from options
  let raw_out: Output = selector.wait_with_output().expect("Failed to read stdout.");
  let select_key: &str = std::str::from_utf8(raw_out.stdout.as_slice()).expect("Failed to convert from &[u8] to &str.");

  // return selection
  if select_key != "" {
    let select_key = select_key.trim_end();
    Some(map.get(select_key).expect("Key not found").clone())
  } else {
    None
  }
}

pub fn screenshot() {
  // Generate HashMap of possible options: (Key: "name of display: e.g. DP-4", Value: "W/mmwxH/mmh+X+Y")
  let mut options: HashMap<String, String> = HashMap::new();
  let raw_out: Output = Command::new("xrandr")
    .args(["--listmonitors"])
    .output()
    .expect("Failed to run xrandr.");
  let raw_out_str: &str = std::str::from_utf8(raw_out.stdout.as_slice()).expect("Failed to convert from &[u8] to &str.");
  let raw_out_vec: Vec<&str> = raw_out_str.split("\n").collect();

  // Get the 2nd column and strip the "+" from the name; get 3rd column and convert from "W/mmwxH/mmh+X+Y" to "WxH+X+Y"
  for monitor in &raw_out_vec[1..raw_out_vec.len() - 1] {
    let monitor_vec: Vec<&str> = monitor.split_whitespace().collect();
    let name: String = monitor_vec[1].strip_prefix("+").expect("Failed to strip prefix.").to_string();
    let crop: String = monitor_vec[2].to_string();
    options.insert(name, crop);
  }

  // Use fancy dmenu & screenshot
  match dmenu(&options) {
    Some(selection) => {
      let date: String = chrono::Local::now().format("%d-%m-%Y_%H-%M-%S").to_string();
      let selection_vec: Vec<&str> = selection.split("/").collect();
      let pos_vec: Vec<&str> = selection_vec[2].split("+").collect();

      let width: &str = selection_vec[0];
      let height: &str = selection_vec[1].split("x").collect::<Vec<&str>>()[1];
      let x: &str = pos_vec[1];
      let y: &str = pos_vec[2];
      let env_home = env::var("HOME").expect("Failed to get HOME.");

      let mut screenshot = Command::new("import")
        .args([
          "-window",
          "root",
          "-crop",
          &format!("{}x{}+{}+{}", width, height, x, y),
          &format!("{}/Downloads/screenshot-{}.png", env_home, date),
        ])
        .spawn()
        .expect("Failed to run import.");

      screenshot.wait().expect("Failed to wait for import.");
    }
    None => {}
  }
}

pub fn char(options: &HashMap<String, String>) {
  // Use fancy dmenu & copy char if supplied
  match dmenu(options) {
    Some(selection) => {
      let mut clip = Command::new("xclip")
        .args(["-in", "-selection", "clipboard"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to run xclip.");

      let mut opt_in = clip.stdin.take().expect("Failed to open stdin.");
      std::thread::spawn(move || {
        opt_in.write_all(selection.as_bytes()).expect("Failed to write to stdin.");
      });

      let _ = clip.wait().expect("Failed to wait for xclip.");
    }
    None => {}
  }
}

pub fn launch(options: &HashMap<String, String>) {
  // Use fancy dmenu & launch output if supplied
  match dmenu(options) {
    Some(selection) => {
      Command::new("sh")
        .args(["-c", &selection])
        .spawn()
        .expect("Failed to launch.");
    }
    None => {}
  }
}
