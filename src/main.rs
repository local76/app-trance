//! trance — Windows Screensaver Manager.
//!
//! Standalone UI for configuring any Windows screensaver.

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unexpected_cfgs)]

#[macro_use]
mod logger;
mod app;
mod backend;
mod bootstrap;
mod bootstrap_guards;
mod chrome;
mod clipboard;
mod config;
mod doctor;
mod theme;
mod ui;
mod utils;
mod win32;
mod win32_relaunch;

#[cfg(test)]
mod tests_perf;

use std::path::PathBuf;
use crate::config::LocalConfig;

#[derive(Debug)]
struct Cli {
    theme: Option<String>,
    command: Option<Command>,
}

#[derive(Debug)]
enum Command {
    Ui,
    Doctor { fix: bool },
}

fn parse_args() -> Result<Cli, String> {
    let mut args = std::env::args().skip(1);
    let mut theme = None;
    let mut cmd_str = None;
    let mut fix = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--theme" => {
                if let Some(val) = args.next() {
                    theme = Some(val);
                } else {
                    return Err("Error: --theme requires a value".into());
                }
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("trance {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "ui" => {
                cmd_str = Some("ui");
            }
            "doctor" => {
                cmd_str = Some("doctor");
            }
            "--fix" => {
                fix = true;
            }
            other => {
                if other.starts_with("--theme=") {
                    theme = Some(other["--theme=".len()..].to_string());
                } else {
                    return Err(format!("Error: unexpected argument '{}'", other));
                }
            }
        }
    }

    let command = match cmd_str {
        Some("ui") => Some(Command::Ui),
        Some("doctor") => Some(Command::Doctor { fix }),
        _ => None,
    };

    Ok(Cli { theme, command })
}

fn print_help() {
    println!("trance — Windows Screensaver Manager");
    println!();
    println!("USAGE:");
    println!("  trance [OPTIONS] [SUBCOMMAND]");
    println!();
    println!("OPTIONS:");
    println!("  --theme <THEME>   Force UI theme: dark, light, high-contrast, no-color");
    println!("  -h, --help        Print help");
    println!("  -V, --version     Print version");
    println!();
    println!("SUBCOMMANDS:");
    println!("  ui                Launch the app dashboard (default)");
    println!("  doctor [--fix]    Check system configuration and diagnostic reports");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("  RUST_LOG  Set log level (error, warn, info, debug, trace)");
    println!("  NO_COLOR  Disable UI color rendering");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    info!("trance start: {:?}", cli);

    let command = cli.command.unwrap_or(Command::Ui);
    let result: Result<(), Box<dyn std::error::Error>> = match command {
        Command::Ui => backend::run_ui(cli.theme.as_deref()),
        Command::Doctor { fix } => doctor::run_doctor(fix),
    };

    if let Err(ref e) = result {
        error!("trance failed: {}", e);
    }
    result
}

