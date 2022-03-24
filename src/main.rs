mod args_parsing;
mod config_parsing;
mod string_parsing;
mod util;

use args_parsing::args_parser::parse_command_line_args;
use config_parsing::config_parser::*;
use log::{debug, error, info, log, trace, warn, Level};
use simple_logger::SimpleLogger;
use std::env::args;
use std::process::exit;
// const NAME_OF_PROGRAM: &'static str = "Alia"; use log::Level;

fn main() {
    if cfg!(debug_assertions) {
        let res = SimpleLogger::new().init();
        if let Err(err) = res {
            println!("Error initializing logger");
            println!("{}", err);
        }
    }
    debug_log!(Level::Info, "Successfully initialized logger");
    debug_debug!("Debug level message");
    debug_error!("Error level message");
    debug_trace!("Trace level message");
    debug_warn!("Warn level message");
    debug_print!("Debug print");
    debug_println!("Debug println");

    let cfg = read_from_config_file();
    if let Err(error) = &cfg {
        println!("Error parsing config!");
        println!("Error:");
        println!("{}", error);
        exit(1);
    }

    let mut cfg = unsafe { cfg.unwrap_unchecked() };
    debug_info!("Successfully parsed config!");
    debug_info!("Config:");
    debug_info!("{:?}", cfg);

    let res = parse_command_line_args(args().collect(), &mut cfg);

    if let Err(e) = res {
        println!("Error parsing your arguments.");
        println!("{}", e);
        exit(1);
    }

    let res = write_to_config_file(&cfg);

    if let Err(_) = res {
        println!("Error writing to cfg file. Your changes may not have been saved.");
        exit(1);
    }
}

fn display_help_message() {
    println!("Available commands are:");
    println!();
    println!("a add ---- Add an alias to Alia ---- Takes name of the alias and the content of the alias as arguments");
    println!(
        "Accepted flags: -i: Alia won't exit on error. -f: Add alias even if it already exists."
    );
    println!("Example usage: alia add run_release \"cargo run --release\"");
    println!("Second example: alias add -i -f run_release \"cargo run --release\"");
    println!("Third example: alias add -if run_release \"cargo run --release\"");
    println!();
    println!("r remove ---- Remove an alias from Alia ---- Takes the name of the alias to remove as an argument");
    println!("Accepted flags: -i: Alia won't exit on error.");
    println!("Example usage: alia remove run_release");
    println!("Second example: alias remove -i run_release");
    println!("c change ---- changes an alias in Alia ---- Takes the name of the alias and the new content as arguments");
    println!(
        "Accepted flags: -i: Alia won't exit on error. -f: change alias even if it doesn't exist."
    );
    println!("Example usage: alia change my_alias \"echo test\"");
    println!("Second example: alias change -i -f my_alias \"echo test\"");
    println!();
    println!("Third example: alias change -if my_alias \"echo test\"");
    println!("e execute ---- Executes the given alias ---- Takes the name of the alias to execute as an argument");
    println!("Accepted flags: -i: Alia won't exit on error.");
    println!("Example usage: alia execute my_alias");
    println!();
    println!("Second example: alias execute -i my_alias");
    println!("h help ---- Displays this message");
    println!();
    println!("Example usage: alia help");
    println!();
    println!();
    println!("Piping together commands is also allowed.");
    println!("For example: alias add my_alias \"echo test\" remove my_alias");
}
