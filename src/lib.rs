#![no_std]

extern crate alloc;

use hashbrown::HashMap;
use alloc::{string::String, vec::Vec, string::ToString, format};

#[derive(Debug, Clone, Default)]
pub struct Command {
    long: String,
    short: String,
    takes_input: bool,
    doc: String,
}


/// A parser for command-line arguments.
///
/// The `Parser` struct provides methods for parsing command-line arguments and extracting key-value pairs.
/// It supports both long and short arguments, as well as flags.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// let mut parser = Parser::new();
/// parser.add_command("name".to_string(), true, 'n');
/// parser.add_command("verbose".to_string(), false, 'v');
///
/// let input = "--name=John --verbose";
/// let result = parser.parse(input.to_string());
///
/// match result {
///     Ok(args) => {
///         println!("Parsed arguments: {:?}", args);
///     },
///     Err(error) => {
///         println!("Error: {}", error);
///     }
/// }
/// ```
pub struct Parser {
    input: String,
    commands: Vec<Command>,
    doc_field: String,
    name: String,
    examples: String,
}

#[derive(Debug, Clone)]
pub struct ParserResult {
    pub map: Option<HashMap<String, String>>,
    pub help: Option<String>,
    pub error: Option<String>,
}

impl ParserResult {
    pub fn map(&self) -> Option<HashMap<String, String>> {
        self.map.clone()
    }
    pub fn help(&self) -> Option<String> {
        self.help.clone()
    }
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }

    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self {
            map: Some(map),
            help: None,
            error: None,
        }
    }
    pub fn from_help(help: String) -> Self {
        Self {
            map: None,
            help: Some(help),
            error: None,
        }
    }
    pub fn from_error(error: String) -> Self {
        Self {
            map: None,
            help: None,
            error: Some(error),
        }
    }
}

impl Parser {
    pub fn new(name: String, doc_field: String, examples: String) -> Self {
        Self {
            input: String::new(),
            commands: Vec::new(),
            doc_field,
            name,
            examples
        }
    }

    pub fn add_command(&mut self, name: String, takes_input: bool, short: String, doc: String) {
        self.commands.push(Command {
            long: name,
            short,
            takes_input,
            doc,
        });
    }

    fn search(&self, arg: &str) -> Option<Command> {
        for command in &self.commands {
            if arg == command.long || arg == command.short {
                return Some(command.clone());
            }
        }
        None
    }

    pub fn parse(&mut self, input: String) -> ParserResult {
        self.input = input;

        let mut args: Vec<String> = Vec::new();

        let mut in_quotes = false;
        let mut cur = String::new();
        for c in self.input.chars() {
            if ['\'', '"'].contains(&c) {
                in_quotes = !in_quotes;
            } else if c == ' ' && !in_quotes {
                if !cur.is_empty() {
                    args.push(cur.clone());
                    cur = String::new();
                }
            } else {
                cur.push(c);
            }
        }
        args.push(cur.clone());

        let mut out = String::new();


        if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
            match args.len() {
                1 => { // global --help
                    out.push_str(format!("Usage: {} [OPTIONS] ...\n\n{}\n", self.name, self.doc_field).as_str());

                    for command in self.commands.clone() {
                        out.push_str(format!("\n  -{} --{}: {} ({})\n", command.short, command.long, command.doc, if command.takes_input { "takes input" } else { "flag" }).as_str());
                    }
                    out.push_str("Examples:\n");
                    for line in self.examples.lines() {
                        out.push_str(format!("    {}\n", line).as_str());
                    }
                    out.push_str("\n");
        
                    return ParserResult::from_help(out);
                },

                2 => { // --help [flag or option]
                    let arg = &args[1];
                    let cmd = self.search(arg);
                    match cmd {
                        Some(command) => {
                            out.push_str(format!(
                                "-{} --{}: {} ({})\n",
                                command.short,
                                command.long,
                                command.doc,
                                if command.takes_input {
                                    "takes input"
                                } else {
                                    "flag"
                                }
                            ).as_str());
                            return ParserResult::from_help(out);
                        },
                        None => {
                            return ParserResult::from_error(format!("Invalid flag/option: {}", arg))
                        }
                    }
                }

                _ => {
                    return ParserResult::from_error("Invalid usage of help flag up top".to_string());
                }
            }

        }

        if !cur.is_empty() {
            args.push(cur);
        }


        let mut result: HashMap<String, String> = HashMap::new();
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];

            if ["-h", "--help"].contains(&arg.as_str()) {
                result.insert("help".to_string(), "present".to_string());
            } else if arg.starts_with("--") {
                let (key, value) = Self::parse_long_arg(arg);

                if value.is_empty() {
                    let cmd = self.search(&key);
                    match cmd {
                        Some(command) => {
                            if command.takes_input {
                                if i + 1 >= args.len() {
                                    return ParserResult::from_error(format!("Invalid argument: {}", arg))
                                } else {
                                    let next_arg = &args[i + 1];
                                    if next_arg.starts_with("--") || next_arg.starts_with("-") {
                                        return ParserResult::from_error(format!("Invalid argument: {}", arg))
                                    }
                                    result.insert(key.to_string(), next_arg.clone());
                                    i += 1;
                                }
                            } else {
                                result.insert(key.to_string(), "present".to_string());
                            }
                        },
                        None => return ParserResult::from_error(format!("Invalid argument: {}", arg))
                    }
                } else {
                    if !self.check(&key) {
                        return ParserResult::from_error(format!("Invalid argument: {}", arg))
                    }
                    result.insert(key.to_string(), value.to_string());
                }
            } else if arg.starts_with("-") {
                let (key, value) = self.parse_short_arg(arg.clone());

                if value.is_empty() {
                    let cmd = self.search(&key);
                    match cmd {
                        Some(command) => {
                            if command.takes_input {
                                if i + 1 >= args.len() {
                                    return ParserResult::from_error(format!("Invalid argument: {}", arg))
                                } else {
                                    let next_arg = &args[i + 1];
                                    if next_arg.starts_with("--") || next_arg.starts_with("-") {
                                        return ParserResult::from_error(format!("Invalid argument: {}", arg))
                                    }
                                    result.insert(key.to_string(), next_arg.clone());
                                    i += 1;
                                }
                            } else {
                                result.insert(key.to_string(), "present".to_string());
                            }
                        },
                        None => return ParserResult::from_error(format!("Invalid argument: {}", arg))
                    }
                } else {
                    if !self.check(&key) {
                        return ParserResult::from_error(format!("Invalid argument: {}", arg))
                    }
                    result.insert(key.to_string(), value.to_string());
                }
            } else {
                let flag = Self::parse_flag(arg);
                if !self.check(&flag) {
                    return ParserResult::from_error(format!("Invalid argument: {}", arg))
                }
                result.insert(flag.to_string(), "present".to_string());
            }

            i += 1;
        }

        if result.contains_key("help") {
            return ParserResult::from_error("Invalid usage of help flag".to_string());
        }

        ParserResult::from_map(result)
    }

    fn parse_flag(arg: &str) -> &str {
        let key = if arg.starts_with("--") {
            &arg[2..]
        } else {
            &arg[1..]
        };
        key
    }

    fn check(&self, arg: &str) -> bool {
        for command in &self.commands {
            if arg == command.long || arg == command.short {
                return true;
            }
        }
        false
    }

    fn parse_long_arg(arg: &str) -> (&str, &str) {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        let key = &parts[0][2..];
        let value = if parts.len() > 1 {
            parts[1]
        } else {
            ""
        };
        (key, value)
    }

    fn parse_short_arg(&self, arg: String) -> (String, String) {
        let key = self.search(&arg[1..=1]).unwrap_or_default().long;
        let value = if arg.len() > 3 {
            arg[3..].to_string()
        } else {
            "".to_string()
        };
        (key, value)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    #[test]
    fn test_parse() {
        let mut tester = Parser::new("test".to_string(), "A test program".to_string(), "test -n=\"John Doe\" --age=20".to_string());
        tester.add_command("name".to_string(), true, "n".to_string(), "The name of the person".to_string());
        tester.add_command("age".to_string(), true, "a".to_string(), "The age of the person".to_string());
        let hash = tester.parse("--help name".to_string());
        std::println!("{:?}", hash);
    }
}
