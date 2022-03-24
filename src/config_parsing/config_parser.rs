use crate::string_parsing::string_parser::{parse_string, StringParseErrorCode};
use crate::util::constants::*;
use crate::util::misc_extension_traits::PopChar;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::{read_to_string, write, File};
#[derive(Debug, Eq, PartialOrd, PartialEq, Ord, Hash, Clone)]
pub enum ConfigParseErrorCode {
    ConfigNotFound,
    ConfigCouldNotBeCreated(String),
    MissingEqualSign(usize),
    MissingAliasValue(usize),
    InvalidAlias(StringParseErrorCode, usize),
    InvalidValue(StringParseErrorCode, usize),
}
use ConfigParseErrorCode::*;
impl Display for ConfigParseErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", config_parse_error_to_string(self))
    }
}

pub fn config_parse_error_to_string(err: &ConfigParseErrorCode) -> String {
    match err {
    ConfigNotFound => format!("A config file could not be found. A new one has been created."),
    ConfigCouldNotBeCreated(s) => format!("A config file could not be found. Creating a new one also failed. Here is the error the OS reported when creating the file:{END_OF_LINE_SEQUENCE}{s}"),
    MissingEqualSign(v) => format!("Expected an equal sign after alias name. Line number: {v}"),
    MissingAliasValue(v) => format!("Missing alias value. Line number: {v}"),
    InvalidAlias(e, v) => format!("The alias name could not be parsed into a string. Line number: {v}. Here is the string parse error:{END_OF_LINE_SEQUENCE}{e}."),
    InvalidValue(e, v) => format!("The alias value could not be parsed into a string. Line number: {v}. Here is the string parse error:{END_OF_LINE_SEQUENCE}{e}."),
  }
}

pub fn parse_config(
    config_as_string: String,
) -> Result<HashMap<String, String>, ConfigParseErrorCode> {
    let slice = &mut config_as_string.trim();
    let mut result = HashMap::with_capacity(config_as_string.lines().count());
    let mut current_alias: usize = 1;
    while slice.len() > 0 {
        let alias = parse_string(slice);
        if let Err(e) = alias {
            return Err(InvalidAlias(e, current_alias));
        }
        let alias = unsafe { alias.unwrap_unchecked() };
        *slice = slice.trim_start();
        let next = slice.pop_char();
        if let None = next {
            return Err(MissingAliasValue(current_alias));
        }
        if next != Some('=') {
            return Err(MissingEqualSign(current_alias));
        }
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

pub fn read_from_config_file() -> Result<HashMap<String, String>, ConfigParseErrorCode> {
    let contents = read_to_string(PATH_TO_CONFIG);
    if let Err(_) = contents {
        if let Err(e) = File::create(PATH_TO_CONFIG) {
            return Err(ConfigCouldNotBeCreated(e.to_string()));
        }
        return Err(ConfigNotFound);
    }
    parse_config(unsafe { contents.unwrap_unchecked() })
}

pub fn config_to_string(cfg: &HashMap<String, String>) -> String {
    let mut result_string = String::with_capacity(cfg.len() * 50);
    for thing in cfg {
        result_string.push('"');
        result_string.push_str(thing.0);
        result_string.push_str("\" = \"");
        result_string.push_str(thing.1);
        result_string.push('"');
        result_string.push_str(END_OF_LINE_SEQUENCE);
    }
    result_string
}

pub fn write_to_config_file(cfg: &HashMap<String, String>) -> Result<(), ()> {
    let result_string = config_to_string(cfg);
    match write("./cfg.alia", result_string) {
        Ok(()) => Ok(()),
        Err(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_parsing::string_parser::StringParseErrorCode::*;
    #[test]
    fn test_parse_cfg() {
        test_parse_cfg_template(
            ["\"my_alias\" = \"echo test\"", "\"test\" = \"echo benis\""],
            Ok([("my_alias", "echo test"), ("test", "echo benis")]),
        );
        test_parse_cfg_template::<1, 1>(
            ["my_alias = echo test"],
            Err(InvalidAlias(StringWithoutOpeningQuote, 1)),
        );
        test_parse_cfg_template::<1, 1>(["\"my_alias = echo test\""], Err(MissingAliasValue(1)));
        test_parse_cfg_template::<1, 1>(["\"my_alias\"\"ttt\""], Err(MissingEqualSign(1)));
        test_parse_cfg_template(
            ["\"my_alias\"=\"t\"\"2\"=\"1\""],
            Ok([("my_alias", "t"), ("2", "1")]),
        );
    }

    fn test_parse_cfg_template<const SIZE_OF_CFG: usize, const SIZE_OF_PARSED_CFG: usize>(
        cfg: [&'static str; SIZE_OF_CFG],
        expected_result: Result<
            [(&'static str, &'static str); SIZE_OF_PARSED_CFG],
            ConfigParseErrorCode,
        >,
    ) {
        let cfg = {
            let mut temp = String::with_capacity(cfg.len() * 50);
            for x in cfg {
                temp.push_str(x);
                temp.push_str(END_OF_LINE_SEQUENCE);
            }
            temp
        };
        let res = parse_config(cfg);
        let expected_result =
            expected_result.map(|x| HashMap::from(x.map(|y| (y.0.to_string(), y.1.to_string()))));
        assert_eq!(res, expected_result);
    }

    #[test]
    fn test_cfg_to_string() {
        test_cfg_to_string_template(
            [("my_alias", "echo test")],
            [format!(
                "\"my_alias\" = \"echo test\"{END_OF_LINE_SEQUENCE}"
            )],
        );
        test_cfg_to_string_template([("my_alias", "echo test"), ("1", "2")], [format!("\"my_alias\" = \"echo test\"{END_OF_LINE_SEQUENCE}\"1\" = \"2\"{END_OF_LINE_SEQUENCE}"), format!("\"1\" = \"2\"{END_OF_LINE_SEQUENCE}\"my_alias\" = \"echo test\"{END_OF_LINE_SEQUENCE}")]);
    }

    fn test_cfg_to_string_template<
        const SIZE_OF_CFG: usize,
        const NUM_OF_ACCEPTABLE_RESULTS: usize,
    >(
        cfg: [(&'static str, &'static str); SIZE_OF_CFG],
        expected_result: [String; NUM_OF_ACCEPTABLE_RESULTS],
    ) {
        let res = config_to_string(&HashMap::from(cfg.map(|x| (x.0.into(), x.1.into()))));
        if expected_result.iter().any(|x| x == &res) {
            return;
        }
        panic!("result {res} did not match with any acceptable result. acceptable results are {expected_result:?}");
    }
}
