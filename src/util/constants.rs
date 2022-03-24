pub const PATH_TO_CONFIG: &'static str = "./cfg.alia";

pub const END_OF_LINE_SEQUENCE: &'static str = if cfg!(windows) { "\r\n" } else { "\n" };

pub const NAME_OF_TERMINAL_PROGRAM: &'static str = if cfg!(windows) { "cmd" } else { "sh" };

pub const RUN_AS_COMMAND_IN_OS: &'static str = if cfg!(windows) { "/C" } else { "-c" };