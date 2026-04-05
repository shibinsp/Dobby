mod display;
mod forge;
mod plan;
mod rule;
mod state;
mod storage;
mod task;
mod tui;

use anyhow::Result;
use display::PREFIX;

fn main() {
    if let Err(err) = entry() {
        eprintln!("{} {}", PREFIX, err);
        std::process::exit(1);
    }
}

fn entry() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        None => tui::run(),
        Some("--forge") => {
            let code = forge::run(&args[2..])?;
            std::process::exit(code);
        }
        Some("plan") => plan::run(&args),
        Some("task") => task::run(&args),
        _ => {
            let code = forge::run(&args[1..])?;
            std::process::exit(code);
        }
    }
}
