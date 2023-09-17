use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn dmenu<'a>(map: HashMap<&str, &'a str>) -> Option<&'a str> {
  // setup dmenu command
  let mut selector = Command::new("dmenu")
    .args(["-c", "-l", "10"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to run dmenu.");

  // pipe in keys
  let mut keys: Vec<&str> = map.keys().map(|key| *key).collect();
  keys.sort();
  let keys_str: String = keys.join("\n");
  let mut opt_in = selector.stdin.take().expect("Failed to open stdin.");
  std::thread::spawn(move || {
    opt_in.write_all(keys_str.as_bytes()).expect("Failed to write to stdin.");
  });

  // extract sanitized value from options
  let raw_out: Output = selector.wait_with_output().expect("Failed to read stdout.");
  let select_key: &str = std::str::from_utf8(raw_out.stdout.as_slice()).expect("Failed to convert from &[u8] to &str.");

  // return selection
  return if select_key != "" {
    Some(&map.get(select_key.strip_suffix("\n").expect("Failed to strip newline.")).expect(""))
  } else {
    None
  }
}

pub fn screenshot() {
  // generate Hashmap of possible options: (Key: "name of display: e.g. DP-4", Value: "WxH+X+Y")
  // example output of xrandr --listmonitors:
  // Monitors: 3
  //  0: +DP-4 2560/527x1440/396+1229+0  DP-4
  //  1: +DP-1 1280/338x1024/270+3789+0  DP-1
  //  2: +DP-2 1229/370x1536/300+0+0  DP-2
  let mut options: HashMap<&str, &str> = HashMap::new();
  let raw_out: Output = Command::new("xrandr").args(["--listmonitors"]).output().expect("Failed to run xrandr.");
  let raw_out_str: &str = std::str::from_utf8(raw_out.stdout.as_slice()).expect("Failed to convert from &[u8] to &str.");
  let raw_out_vec: Vec<&str> = raw_out_str.split("\n").collect();
  // get the 2nd column and strip the "+" from the name; get 3rd column and convert from "W/mmwxH/mmh+X+Y" to "WxH+X+Y"
  for monitor in &raw_out_vec[1..raw_out_vec.len()-1] {
    let monitor_vec: Vec<&str> = monitor.split_whitespace().collect();
    let name: &str = monitor_vec[1].strip_prefix("+").expect("Failed to strip prefix.");
    let crop: &str = monitor_vec[2];
    options.insert(name, crop);
  }

  // use fancy dmenu & screenshot
  match dmenu(options) {
    Some(selection) => {
      let date: String = chrono::Local::now().format("%d-%m-%Y_%H-%M-%S").to_string();
      let selection_vec: Vec<&str> = selection.split("/").collect();
      let pos_vec: Vec<&str> = selection_vec[2].split("+").collect();

      let width: &str = selection_vec[0];
      let height: &str = selection_vec[1].split("x").collect::<Vec<&str>>()[1];
      let x: &str = pos_vec[1];
      let y: &str = pos_vec[2];
      let mut screenshot = Command::new("import")
        .args(["-window", "root", "-crop", &format!("{}x{}+{}+{}",width, height, x, y), &format!("/home/philip/Downloads/screenshot-{}.png", date)])
        .spawn()
        .expect("Failed to run import.");
      screenshot.wait().expect("Failed to wait for import.");
    }
    None => {  }
  }
}

pub fn char() {
  // Hashmap of possible options: (Key: "en_US char", Value: "special character")
  let options: HashMap<&str, &str> = HashMap::from([
    ("a", "ä"),
    ("A", "Ä"),
    ("o", "ö"),
    ("O", "Ö"),
    ("u", "ü"),
    ("U", "Ü"),
    ("sz", "ß"),
    ("SZ", "ẞ"),
  ]);

  // use fancy dmenu & copy char if supplied
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
      let raw_out: Output = clip.wait_with_output().expect("Failed to read stdout.");
      let feedback: &str = std::str::from_utf8(raw_out.stdout.as_slice()).expect("Failed to convert from &[u8] to &str.");
      println!("{}", feedback);
    }
    None => {  }
  }

}

pub fn launch_script() {
  // Hashmap of possible options: (Key: "command name", Value: "command")
  let options: HashMap<&str, &str> = HashMap::from([
    ("kill Xorg", "pkill Xorg"),
    ("kill chromium", "pkill chromium"),
    ("manage bD", "betterdiscord-installer"),
    ("poweroff", "sudo poweroff"),
    ("reboot", "sudo reboot"),
    ("run", "dmenu_run -c -l 5"),
    ("screenshot", "yah g"),
    ("sleep", "sudo zzz"),
    ("standby", "xset dpms force suspend"),
  ]);

  // use fancy dmenu & launch output if supplied
  match dmenu(options) {
    Some(selection) => { Command::new("sh").args(["-c", selection]).output().expect("Failed to launch."); }
    None => {  }
  }
}

pub fn launch_application() {
  // Hashmap of possible options: (Key: "program", Value: "launch command")
  let options: HashMap<&str, &str> = HashMap::from([
    ("arandr", "arandr"),
    ("bleachbit", "bleachbit"),
    ("chromium", "chromium"),
    ("clion", "/home/philip/.local/share/JetBrains/Toolbox/scripts/clion"),
    ("datagrip", "/home/philip/.local/share/JetBrains/Toolbox/scripts/datagrip"),
    ("discord", "discord"),
    ("gimp", "gimp"),
    ("inkscape", "inkscape"),
    ("lxappearance", "lxappearance"),
    ("nvidia", "nvidia-settings"),
    ("office", "libreoffice"),
    ("piper", "piper"),
    ("prismlauncher", "prismlauncher"),
    ("pycharm", "/home/philip/.local/share/JetBrains/Toolbox/scripts/pycharm"),
    ("spotify", "spotify"),
    ("toolbox", "jetbrains-toolbox"),
    ("tor", "tor-browser"),
    ("whatsapp", "whatsapp-for-linux"),
  ]);

  // use fancy dmenu & launch output if supplied
  match dmenu(options) {
    Some(selection) => { Command::new(selection).spawn().expect("Failed to launch."); }
    None => {  }
  }
}
