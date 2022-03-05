use std::collections::HashMap;
use std::{fs};
use std::fmt::{Display, Formatter};
use std::fs::{File};
use std::process::{Command, exit};
use simple_logger::SimpleLogger;
use std::env::{args};
use std::hash::Hash;


trait OkOrDefault {
  type T;
  fn ok_or_default<E: Default>(self) -> Result<Self::T, E>;
}

impl<X> OkOrDefault for Option<X> {
  type T = X;
  fn ok_or_default<E: Default>(self) -> Result<Self::T, E> {
    self.ok_or(E::default())
  }
}


const PATH_TO_CONFIG: &'static str = "./cfg.alia";

const END_OF_LINE_SEQUENCE: &'static str = if cfg!(windows) { "\r\n" } else { "\n" };

const NAME_OF_TERMINAL_PROGRAM: &'static str = if cfg!(windows) { "cmd" } else { "sh" };
const RUN_AS_COMMAND_IN_OS: &'static str = if cfg!(windows) { "/C" } else { "-c" };


use log::{info, log, Level};

#[derive(Debug, Eq, PartialOrd, PartialEq, Ord, Hash, Clone)]
enum StringParseErrorCode {
  StringWithoutOpeningQuote,
  InvalidString,
  EmptyString,
  StringWithoutClosingQuote,
}

impl Display for StringParseErrorCode {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", string_parse_error_to_string(self))
  }
}

fn string_parse_error_to_string(err: &StringParseErrorCode) -> String {
  match err {
    StringWithoutOpeningQuote => "Your string did not contain an opening quote.",
    InvalidString => "Your string consisted of only an opening parenthesis and nothing else.",
    EmptyString => "Your string was empty. This is not allowed as all aliases must have names and values that are not empty.",
    StringWithoutClosingQuote => "Your string did not contain a closing quote.",
  }.to_string()
}

#[derive(Debug, Eq, PartialOrd, PartialEq, Ord, Hash, Clone)]
enum ConfigParseErrorCode {
  ConfigNotFound,
  ConfigCouldNotBeCreated(String),
  MissingEqualSign(u32),
  MissingAliasValue(u32),
  InvalidAlias(StringParseErrorCode, u32),
  InvalidValue(StringParseErrorCode, u32),
}

impl Display for ConfigParseErrorCode {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", config_parse_error_to_string(self))
  }
}

fn config_parse_error_to_string(err: &ConfigParseErrorCode) -> String {
  match err {
    ConfigNotFound => format!("A config file could not be found. A new one has been created."),
    ConfigCouldNotBeCreated(s) => format!("A config file could not be found. Creating a new one also failed. Here is the error the OS reported when creating the file:{END_OF_LINE_SEQUENCE}{s}"),
    MissingEqualSign(v) => format!("Expected an equal sign after alias name. Line number: {v}"),
    MissingAliasValue(v) => format!("Missing alias value. Line number: {v}"),
    InvalidAlias(e, v) => format!("The alias name could not be parsed into a string. Line number: {v}. Here is the string parse error:{END_OF_LINE_SEQUENCE}{e}."),
    InvalidValue(e, v) => format!("The alias value could not be parsed into a string. Line number: {v}. Here is the string parse error:{END_OF_LINE_SEQUENCE}{e}."),
  }
}

#[derive(Debug, Eq, PartialOrd, PartialEq, Ord, Hash, Clone)]
enum CommandLineArgumentErrorCode {
  MissingNameArgument(u32),
  MissingContentArgument(u32),
  FailedExecute(u32),
  CannotRemoveNonExistentValue(String, u32),
  InvalidAliasName(String, u32),
  AliasDoesNotExist(String, u32),
  AliasAlreadyExists(String, u32),
  InvalidCommand(String, u32),
  NoValidArgs(u32),
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
    FailedExecute(v) => format!("Execution of execute command failed. Error occurred at argument number {v}"),
    CannotRemoveNonExistentValue(name, v) => format!("Error removing alias of name {name}. This alias had no value associated with it and thus could not be removed. Error occurred at argument number {v}"),
    InvalidAliasName(name, v) => format!("Alias with name {name} has no content associated with it and thus cannot be executed. Error occurred at argument number {v}"),
    AliasDoesNotExist(name, v) => format!("Cannot change alias with name {name} because it does not exist. Error occurred at argument number {v}"),
    AliasAlreadyExists(name, v) => format!("Cannot create alias with name {name} that already exists. Error occurred at argument number {v}"),
    InvalidCommand(name, v) => format!("Command with name {name} does not exist. Error occurred at argument number {v}"),
    NoValidArgs(v) => format!("You passed no valid arguments to Alia. Error occurred at argument number {v}"),
  }
}

macro_rules! debug_log {
  (target: $target:expr, $lvl:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
        log!($target, $lvl, $($arg)+);
    }
  );
  ($lvl:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
      log!($lvl, $($arg)+);
    }
  );
}

macro_rules! debug_info {
  (target: $target:expr, $($arg:tt)+) => (
    if cfg!(debug_assertions) {
      info!($target, $($arg)+);
    }
  );
  ($($arg:tt)+) => (
    if cfg!(debug_assertions) {
      info!($($arg)+);
    }
  );
}
macro_rules! printlnln {
  ($($arg:tt)+) => {
    println!($($arg)+);
    println!();
  };
  () => {
    println!();
    println!();
  };
}

macro_rules! exit_failure {
  () => {
    exit(1);
  };
}
use StringParseErrorCode::*;
use crate::ConfigParseErrorCode::*;
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

  let cfg = read_from_config_file();
  if let Err(error) = &cfg {
    println!("Error parsing config!");
    println!("Error:");
    println!("{}", error);
    exit_failure!();
  }

  let mut cfg = unsafe { cfg.unwrap_unchecked() };
  debug_info!("Successfully parsed config!");
  debug_info!("Config:");
  debug_info!("{:?}", cfg);


  let res = parse_command_line_args(args(), &mut cfg);

  if let Err(e) = res {
    println!("Error parsing your arguments.");
    println!("{}", e);
    exit_failure!();
  }

  let res = write_to_config_file(&cfg);

  if let Err(_) = res {
    println!("Error writing to cfg file. Your changes may not have been saved.");
    exit_failure!();
  }
}

type CommandLineArgParser<T> = &'static dyn Fn(&mut T, &mut u32, &mut HashMap<String, String>) -> Result<(), CommandLineArgumentErrorCode>;

fn get_next_arg<T: ExactSizeIterator<Item = String>>(args: &mut T, current_arg: &mut u32) -> Option<String> {
  let res = args.next()?;
  *current_arg += 1;
  Some(res)
}

fn parse_command_line_args<T: 'static + ExactSizeIterator<Item = String>>(mut args: T, cfg: &mut HashMap<String, String>) -> Result<(), CommandLineArgumentErrorCode> {
  let mut current_arg: u32 = 0;
  let maybe_path = args.next();
  let mut first_arg_invalid = false;
  if let Some(arg) = maybe_path {
    current_arg += 1;
    let parser = parse_arg(arg.as_str()).unwrap_or_else(|| {
      first_arg_invalid = true;
      &do_nothing
    });
    parser(&mut args, &mut current_arg, cfg)?;
  }
  if args.len() == 0 && first_arg_invalid {
    return Err(NoValidArgs(current_arg));
  }
  while let Some(arg) = args.next() {
    current_arg += 1;
    let arg_str = arg.as_str();
    let parser = parse_arg::<T>(arg_str);
    if let None = parser {
      return Err(InvalidCommand(arg, current_arg));
    }
    let parser = unsafe { parser.unwrap_unchecked() };
    parser(&mut args, &mut current_arg, cfg)?;
  }
  Ok(())
}

fn parse_arg<T: ExactSizeIterator<Item = String>>(arg: &str) -> Option<CommandLineArgParser<T>> {
  let res: CommandLineArgParser<T> = match arg {
    "-a" | "--add" => &add_alias,
    "-r" | "--remove" => &remove_alias,
    "-e" | "--execute" => &execute_alias,
    "-c" | "--change" => &change_alias,
    "-h" | "--help" => {
      display_help_message();
      &do_nothing
    }
    _ => { return None; }
  };
  Some(res)
}

fn do_nothing<T: ExactSizeIterator<Item = String>>(_args: &mut T, _current_arg: &mut u32, _cfg: &mut HashMap<String, String>) -> Result<(), CommandLineArgumentErrorCode> { Ok(()) }

fn add_alias<T: ExactSizeIterator<Item = String>>(args: &mut T, current_arg: &mut u32, cfg: &mut HashMap<String, String>) -> Result<(), CommandLineArgumentErrorCode> {
  let name_of_alias = get_next_arg(args, current_arg).ok_or(MissingNameArgument(*current_arg))?;
  let content_of_alias = get_next_arg(args, current_arg).ok_or(MissingContentArgument(*current_arg))?;
  let res = cfg.get(name_of_alias.as_str());
  if let Some(_) = res {
    return Err(AliasAlreadyExists(name_of_alias, *current_arg));
  }
  cfg.insert(name_of_alias, content_of_alias);
  Ok(())
}

fn remove_alias<T: ExactSizeIterator<Item = String>>(args: &mut T, current_arg: &mut u32, cfg: &mut HashMap<String, String>) -> Result<(), CommandLineArgumentErrorCode> {
  let name_of_alias = get_next_arg(args, current_arg).ok_or(MissingNameArgument(*current_arg))?;
  match cfg.remove(&name_of_alias) {
    Some(_) => Ok(()),
    None => Err(CannotRemoveNonExistentValue(name_of_alias, *current_arg))
  }
}

fn execute_alias<T: ExactSizeIterator<Item = String>>(args: &mut T, current_arg: &mut u32, cfg: &mut HashMap<String, String>) -> Result<(), CommandLineArgumentErrorCode> {
  let name_of_alias = get_next_arg(args, current_arg).ok_or(MissingNameArgument(*current_arg))?;
  let content_of_alias = cfg.get(name_of_alias.as_str()).ok_or(InvalidAliasName(name_of_alias, *current_arg))?;
  let split = content_of_alias.split_whitespace();
  let iter = [RUN_AS_COMMAND_IN_OS].iter().map(|x| *x);
  let iter = iter.chain(split);
  let output = Command::new(NAME_OF_TERMINAL_PROGRAM).args(iter).spawn();
  match output {
    Ok(_) => Ok(()),
    Err(_) => Err(FailedExecute(*current_arg))
  }
}

fn change_alias<T: ExactSizeIterator<Item = String>>(args: &mut T, current_arg: &mut u32, cfg: &mut HashMap<String, String>) -> Result<(), CommandLineArgumentErrorCode> {
  let name_of_alias = get_next_arg(args, current_arg).ok_or(MissingNameArgument(*current_arg))?;
  let content_of_alias = get_next_arg(args, current_arg).ok_or(MissingContentArgument(*current_arg))?;


  let res = cfg.get(name_of_alias.as_str());
  if let None = res {
    return Err(AliasDoesNotExist(name_of_alias, *current_arg));
  }
  cfg.insert(name_of_alias, content_of_alias);
  Ok(())
}

fn read_from_config_file() -> Result<HashMap<String, String>, ConfigParseErrorCode> {
  let contents = fs::read_to_string(PATH_TO_CONFIG);
  if let Err(_) = contents {
    if let Err(e) = File::create(PATH_TO_CONFIG) {
      return Err(ConfigCouldNotBeCreated(e.to_string()));
    }
    return Err(ConfigNotFound);
  }
  let contents = unsafe { contents.unwrap_unchecked() };
  let slice = &mut contents.trim();
  let mut result = HashMap::with_capacity(contents.lines().count());
  let mut current_alias: u32 = 1;
  while slice.len() > 0 {
    let alias = parse_string(slice);
    if let Err(e) = alias {
      return Err(InvalidAlias(e, current_alias));
    }
    let alias = unsafe { alias.unwrap_unchecked() };
    *slice = slice.trim_start();
    let next = slice.chars().next();
    if let None = next {
      return Err(MissingAliasValue(current_alias));
    }
    if next != Some('=') {
      return Err(MissingEqualSign(current_alias));
    }
    *slice = &slice[1..];
    let value = parse_string(slice);
    if let Err(e) = value {
      return Err(InvalidValue(e, current_alias));
    }
    let value = unsafe { value.unwrap_unchecked() };
    result.insert(alias, value);
    *slice = &slice.trim_start();
    current_alias += 1;
  }
  Ok(result)
}

fn write_to_config_file(cfg: &HashMap<String, String>) -> Result<(), ()> {
  let mut result_string = String::with_capacity(cfg.len() * 50);
  for thing in cfg {
    result_string.push('"');
    result_string.push_str(thing.0);
    result_string.push_str("\" = \"");
    result_string.push_str(thing.1);
    result_string.push('"');
    result_string.push_str(END_OF_LINE_SEQUENCE);
  }
  match fs::write("./cfg.alia", result_string) {
    Ok(()) => Ok(()),
    Err(_) => Err(()),
  }
}

fn parse_string(slice: &mut &str) -> Result<String, StringParseErrorCode> {
  parse_string_track_lines(slice, &mut 0)
}

fn parse_string_track_lines(slice: &mut &str, current_line: &mut u32) -> Result<String, StringParseErrorCode> {
  *slice = slice.trim_start();
  if slice.chars().next() != Some('"') {
    return Err(StringWithoutOpeningQuote);
  }
  *slice = &slice[1..];
  let ret_slice: &str = *slice;
  let current_char = slice.chars().next();
  if let None = current_char {
    return Err(InvalidString);
  }
  let mut current_char = unsafe { current_char.unwrap_unchecked() };
  let mut index = 1;
  let mut backslashes_in_a_row: usize = 0;
  loop {
    if current_char == '\n' {
      *current_line += 1;
    }
    if current_char == '"' && backslashes_in_a_row % 2 == 0 {
      *slice = &slice[1..];

      break;
    }
    if current_char == '\\' {
      backslashes_in_a_row += 1;
    } else {
      backslashes_in_a_row = 0;
    }
    *slice = &slice[1..];
    let current_char_option = slice.chars().next();
    if let None = current_char_option {
      return Err(StringWithoutClosingQuote);
    }
    current_char = unsafe { current_char_option.unwrap_unchecked() };
    index += 1;
  }
  let mut ret = String::with_capacity(index - 1);
  let mut iter = ret_slice.chars();
  let mut last_char = '\0';

  for _ in 0..index - 1 {
    let char = unsafe { iter.next().unwrap_unchecked() };
    if last_char == '\\' && char == '"' {
      ret.pop();
    }
    ret.push(char);
    last_char = char;
  }

  if ret.len() == 0 {
    return Err(EmptyString);
  }
  ret.shrink_to_fit();
  Ok(ret)
}

fn display_help_message() {
  printlnln!("Available commands are:");
  println!("-a --add ---- Add an alias to Alia ---- Takes name of the alias and the content of the alias as arguments");
  printlnln!("Example usage: alia --add run_release \"cargo run --release\"");
  println!("-r --remove ---- Remove an alias from Alia ---- Takes the name of the alias to remove as an argument");
  printlnln!("Example usage: alia --remove run_release");
  println!("-c --change ---- changes an alias in Alia ---- Takes the name of the alias and the new content as arguments");
  printlnln!("Example usage: alia --change my_alias \"echo test\"");
  println!("-e --execute ---- Executes the given alias ---- Takes the name of the alias to execute as an argument");
  printlnln!("Example usage: alia --execute my_alias");
  println!("-h --help ---- Displays this message");
  printlnln!("Example usage: alia --help");
}


#[cfg(test)]
mod tests {
  #![allow(warnings)]

  use crate::{parse_string, StringParseErrorCode};
  use StringParseErrorCode::*;

  #[test]
  fn test_parse_string() {
    assert_eq!(parse_string(&mut "dsasd\"dsadaasdfdsd\"dsaadsasd"), Err(StringWithoutOpeningQuote));
    assert_eq!(parse_string(&mut "\"Snail\"abcdef\"PENIS\""), Ok("Snail".to_string()));
    assert_eq!(parse_string(&mut "\"Snail\""), Ok("Snail".to_string()));
    assert_eq!(parse_string(&mut "\"S\\\"nail\""), Ok("S\"nail".to_string()));
    assert_eq!(parse_string(&mut "\"SSS"), Err(StringWithoutClosingQuote));


    let string = String::from("\"Snail\"");
    let slice = &mut string.as_str();
    let parsed = parse_string(slice);
    assert_eq!(parsed, Ok("Snail".into()));
    assert_eq!(slice, &"");
    util("\"Snail\"", "", Ok("Snail".into()));
    util("\"HAAAAAA\"some more stuff", "some more stuff", Ok("HAAAAAA".into()));
  }

  fn util(input: &'static str, expected_slice_at_end: &'static str, expected_result: Result<String, StringParseErrorCode>) {
    let string = String::from(input);
    let slice = &mut string.as_str();
    let parsed = parse_string(slice);
    assert_eq!(parsed, expected_result);
    assert_eq!(slice, &expected_slice_at_end);
  }
}