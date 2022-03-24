use crate::display_help_message;
use crate::util;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::process::Command;
use util::constants::*;
use util::misc_extension_traits::{Peek, Pop, PopChar};
#[derive(Debug, Eq, PartialOrd, PartialEq, Ord, Hash, Clone)]
pub enum CommandLineArgumentErrorCode {
    MissingNameArgument(usize),
    MissingContentArgument(usize),
    FailedExecute(String, usize),
    CannotRemoveNonExistentValue(String, usize),
    InvalidAliasName(String, usize),
    AliasDoesNotExist(String, usize),
    AliasAlreadyExists(String, usize),
    InvalidCommand(String, usize),
    NoValidArgs,
    NoArgs,
}

use CommandLineArgumentErrorCode::*;

impl Display for CommandLineArgumentErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", command_line_error_to_string(self))
    }
}

fn command_line_error_to_string(err: &CommandLineArgumentErrorCode) -> String {
    match err {
    MissingNameArgument(v) => format!("You did not supply a name argument for command. Error occurred at argument number {v}"),
    MissingContentArgument(v) => format!("You did not supply a content argument for command. Error occurred at argument number {v}"),
    FailedExecute(error, v) => format!("Execution of execute command failed. Error occurred at argument number {v}. Here is the error the OS returned.{END_OF_LINE_SEQUENCE}{END_OF_LINE_SEQUENCE}{error}"),
    CannotRemoveNonExistentValue(name, v) => format!("Error removing alias of name {name}. This alias had no value associated with it and thus could not be removed. Error occurred at argument number {v}"),
    InvalidAliasName(name, v) => format!("Alias with name {name} has no content associated with it and thus cannot be executed. Error occurred at argument number {v}"),
    AliasDoesNotExist(name, v) => format!("Cannot change alias with name {name} because it does not exist. Error occurred at argument number {v}"),
    AliasAlreadyExists(name, v) => format!("Cannot create alias with name {name} that already exists. Error occurred at argument number {v}"),
    InvalidCommand(name, v) => format!("Command with name {name} does not exist. Error occurred at argument number {v}"),
    NoValidArgs => format!("You passed no valid arguments to Alia."),
    NoArgs => format!("You did not pass any args to Alia."),
  }
}

pub type CommandLineArgParser = &'static dyn Fn(
    &mut Vec<String>,
    &mut usize,
    &mut HashMap<String, String>,
) -> Result<(), CommandLineArgumentErrorCode>;

fn get_next_arg<'a>(args: &'a mut Vec<String>, current_arg: &mut usize) -> Option<String> {
    let res = args.pop()?;
    *current_arg += 1;
    Some(res)
}

//fn peek_next_arg(args: &mut Vec<String>) -> Option<&str> {
//    Some(args.get(0)?.as_str())
//}

pub fn parse_command_line_args(
    mut args: Vec<String>,
    cfg: &mut HashMap<String, String>,
) -> Result<(), CommandLineArgumentErrorCode> {
    args.reverse();
    let mut current_arg: usize = 0;
    if args.len() == 0 {
        return Err(NoArgs);
    }
    let maybe_path = args.pop();
    let mut first_arg_invalid = false;
    if let Some(arg) = maybe_path {
        let parser = parse_arg(arg.as_str()).unwrap_or_else(|| {
            first_arg_invalid = true;
            &do_nothing
        });
        if !first_arg_invalid {
            current_arg += 1;
        }
        parser(&mut args, &mut current_arg, cfg)?;
    }
    if args.len() == 0 && first_arg_invalid {
        return Err(NoValidArgs);
    }
    while let Some(arg) = args.pop() {
        current_arg += 1;
        let arg_str = arg.as_str();
        let parser = parse_arg(arg_str);
        if let None = parser {
            return Err(InvalidCommand(arg, current_arg));
        }
        let parser = unsafe { parser.unwrap_unchecked() };
        parser(&mut args, &mut current_arg, cfg)?;
    }
    Ok(())
}

struct Flags {
    force: bool,
    ignore_errors: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            force: false,
            ignore_errors: false,
        }
    }
}

enum ParseFlagsErrorCode {
    FlagAlreadySet(&'static str, usize),
}

fn parse_flags(
    args: &mut Vec<String>,
    current_arg: &mut usize,
) -> Result<Flags, ParseFlagsErrorCode> {
    let mut flags = Flags::default();

    while args.peek().is_some() && unsafe { args.peek_unchecked() }.starts_with("-") {
        *current_arg += 1;
        let arg_being_parsed = unsafe { args.pop_unchecked() };
        let mut arg_being_parsed_as_str = arg_being_parsed.as_str();
        arg_being_parsed_as_str.pop_char();
        for char in arg_being_parsed_as_str.bytes() {
            handle_flag(
                char,
                b'i',
                &mut flags.ignore_errors,
                "Ignore errors",
                *current_arg,
            )?;
            handle_flag(char, b'f', &mut flags.force, "Force", *current_arg)?;
        }
    }

    Ok(flags)
}

fn handle_flag(
    current_char: u8,
    char_to_match: u8,
    flag: &mut bool,
    flag_as_str: &'static str,
    current_arg: usize,
) -> Result<(), ParseFlagsErrorCode> {
    if current_char == char_to_match {
        if *flag {
            return Err(ParseFlagsErrorCode::FlagAlreadySet(
                flag_as_str,
                current_arg,
            ));
        }
        *flag = true;
    }
    Ok(())
}

fn parse_arg(arg: &str) -> Option<CommandLineArgParser> {
    let res: CommandLineArgParser = match arg {
        "a" | "add" => &add_alias,
        "r" | "remove" => &remove_alias,
        "e" | "execute" => &execute_alias,
        "c" | "change" => &change_alias,
        "h" | "help" => {
            display_help_message();
            &do_nothing
        }
        _ => {
            return None;
        }
    };
    Some(res)
}

fn do_nothing(
    _args: &mut Vec<String>,
    _current_arg: &mut usize,
    _cfg: &mut HashMap<String, String>,
) -> Result<(), CommandLineArgumentErrorCode> {
    Ok(())
}

fn add_alias(
    args: &mut Vec<String>,
    current_arg: &mut usize,
    cfg: &mut HashMap<String, String>,
) -> Result<(), CommandLineArgumentErrorCode> {
    let name_of_alias = get_next_arg(args, current_arg).ok_or(MissingNameArgument(*current_arg))?;
    let res = cfg.get(name_of_alias.as_str());
    if let Some(_) = res {
        return Err(AliasAlreadyExists(name_of_alias, *current_arg));
    }
    let content_of_alias =
        get_next_arg(args, current_arg).ok_or(MissingContentArgument(*current_arg))?;
    cfg.insert(name_of_alias, content_of_alias);
    Ok(())
}

fn remove_alias(
    args: &mut Vec<String>,
    current_arg: &mut usize,
    cfg: &mut HashMap<String, String>,
) -> Result<(), CommandLineArgumentErrorCode> {
    let name_of_alias = get_next_arg(args, current_arg).ok_or(MissingNameArgument(*current_arg))?;
    match cfg.remove(&name_of_alias) {
        Some(_) => Ok(()),
        None => Err(CannotRemoveNonExistentValue(name_of_alias, *current_arg)),
    }
}

fn execute_alias(
    args: &mut Vec<String>,
    current_arg: &mut usize,
    cfg: &mut HashMap<String, String>,
) -> Result<(), CommandLineArgumentErrorCode> {
    let name_of_alias = get_next_arg(args, current_arg).ok_or(MissingNameArgument(*current_arg))?;
    let content_of_alias = cfg
        .get(name_of_alias.as_str())
        .ok_or(InvalidAliasName(name_of_alias, *current_arg))?;
    let split = content_of_alias.split_whitespace();
    let iter = [RUN_AS_COMMAND_IN_OS].into_iter();
    let iter = iter.chain(split);
    let output = Command::new(NAME_OF_TERMINAL_PROGRAM).args(iter).spawn();
    match output {
        Ok(_) => Ok(()),
        Err(e) => Err(FailedExecute(e.to_string(), *current_arg)),
    }
}

fn change_alias(
    args: &mut Vec<String>,
    current_arg: &mut usize,
    cfg: &mut HashMap<String, String>,
) -> Result<(), CommandLineArgumentErrorCode> {
    let name_of_alias = get_next_arg(args, current_arg).ok_or(MissingNameArgument(*current_arg))?;
    let res = cfg.get(name_of_alias.as_str());
    if let None = res {
        return Err(AliasDoesNotExist(name_of_alias, *current_arg));
    }
    let content_of_alias =
        get_next_arg(args, current_arg).ok_or(MissingContentArgument(*current_arg))?;
    cfg.insert(name_of_alias, content_of_alias);
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::{parse_command_line_args, CommandLineArgumentErrorCode};
    use std::collections::HashMap;
    use CommandLineArgumentErrorCode::*;

    #[test]
    fn test_parse_command_line_args() {
        test_cmd_args_no_cfg(["help"], Ok(()));
        test_cmd_args_no_cfg([""], Err(NoValidArgs));
        test_cmd_args_no_cfg([], Err(NoArgs));
        test_cmd_args_no_cfg(["h", "add", "my_alias", "echo test"], Ok(()));
        test_cmd_args_template(
            ["a", "my_alias", "echo test"],
            [],
            Ok(()),
            [("my_alias", "echo test")],
        );
        test_cmd_args_template(
            ["a", "my_alias", "echo test", "r", "my_alias"],
            [],
            Ok(()),
            [],
        );
        test_cmd_args_template(
            ["c", "my_alias", "echo test"],
            [],
            Err(AliasDoesNotExist("my_alias".to_string(), 2)),
            [],
        );
        test_cmd_args_template(
            ["e", "my_alias"],
            [],
            Err(InvalidAliasName("my_alias".to_string(), 2)),
            [],
        );
        test_cmd_args_template(
            ["a", "my_alias", "echo test", "e", "my_alias"],
            [],
            Ok(()),
            [("my_alias", "echo test")],
        );
    }

    fn test_cmd_args_no_cfg<const NUM_OF_ARGS: usize>(
        args: [&'static str; NUM_OF_ARGS],
        expected_result: Result<(), CommandLineArgumentErrorCode>,
    ) {
        println!("Entered test_cmd_args_no_cfg with args : {args:?}");
        let res = parse_command_line_args(
            args.into_iter().map(|x| x.to_string()).collect(),
            &mut HashMap::new(),
        );
        assert_eq!(res, expected_result);
    }

    fn test_cmd_args_template<
        const NUM_OF_ARGS: usize,
        const SIZE_OF_CFG: usize,
        const SIZE_OF_CFG_AFTER: usize,
    >(
        args: [&'static str; NUM_OF_ARGS],
        cfg: [(&'static str, &'static str); SIZE_OF_CFG],
        expected_result: Result<(), CommandLineArgumentErrorCode>,
        cfg_after: [(&'static str, &'static str); SIZE_OF_CFG_AFTER],
    ) {
        let mut cfg: HashMap<String, String> = cfg
            .into_iter()
            .map(|x| (x.0.to_string(), x.1.to_string()))
            .collect();
        let cfg_after: HashMap<String, String> = cfg_after
            .into_iter()
            .map(|x| (x.0.to_string(), x.1.to_string()))
            .collect();
        println!("Entered test_cmd_args_template with args : {args:?}");
        let res =
            parse_command_line_args(args.into_iter().map(|x| x.to_string()).collect(), &mut cfg);
        assert_eq!(res, expected_result);
        assert_eq!(cfg, cfg_after);
    }
}
