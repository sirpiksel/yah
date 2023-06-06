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
    ("kill Whatsapp", "pkill WhatsApp"),
    ("kill Xorg", "pkill Xorg"),
    ("kill chromium", "pkill chromium"),
    ("poweroff", "sudo poweroff"),
    ("reboot", "sudo reboot"),
    ("run", "dmenu_run -c -l 5"),
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
    ("tor", "tor-browser"),
    ("whatsapp", "whatsapp-nativefier"),
    ("toolbox", "jetbrains-toolbox"),
  ]);

  // use fancy dmenu & launch output if supplied
  match dmenu(options) {
    Some(selection) => { Command::new(selection).spawn().expect("Failed to launch."); }
    None => {  }
  }
}
