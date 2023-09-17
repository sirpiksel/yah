use std::env;
use std::collections::VecDeque;

mod util;

fn main() {
  // argument setup
  let args: Vec<String> = env::args().collect();
  let mut arg_depth: u8 = 0;
  let mut args_sane: VecDeque<&str> = VecDeque::new();
  let mut help: bool = false;
  let mut error: bool = false;

  // help output string
  const HELP: &'static str = "Usage: yah [OPTION]...
  Options:
  h, help         print help
  a, appLaunch    start Application-Launcher
  s, scrLaunch    start Script-Launcher
  c, char         start Charpicker
  g, grab         start Screenshot-Tool";

  // process arguments
  for arg in args.iter() {
    if arg != &args[0] { // skip if executable
      match arg.as_str() {
        "h" | "help" => { help = true; break;}
        "a" | "appLaunch" => {
          if arg_depth == 0 {
            arg_depth += 1;
            args_sane.push_back("appLaunch");
          } else {
            error = true; break;
          }
        }
        "s" | "scrLaunch" => {
          if arg_depth == 0 {
            arg_depth += 1;
            args_sane.push_back("scrLaunch");
          } else {
            error = true; break;
          }
        }
        "c" | "char" => {
          if arg_depth == 0 {
            arg_depth += 1;
            args_sane.push_back("char");
          } else {
            error = true; break;
          }
        }
        "g" | "grab" => {
          if arg_depth == 0 {
            arg_depth += 1;
            args_sane.push_back("grab");
          } else {
            error = true; break;
          }
        }
        _ => { error = true; break; }
      }
    }
  }

  // run functionality
  if error {
    eprintln!("{}", HELP);
  } else if help || args_sane.len() == 0 {
    println!("{}", HELP);
  } else {
    match args_sane.pop_front() {
      Some("appLaunch") => { util::launcher::launch_application(); }
      Some("scrLaunch") => { util::launcher::launch_script(); }
      Some("char") => { util::launcher::char(); }
      Some("grab") => { util::launcher::screenshot(); }
      Some(_) => { panic!("invalid element in args_sane"); }
      None => { panic!("no elements in args_sane"); }
    }
  }
}
