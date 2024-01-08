#![no_std]

extern crate alloc;

use hashbrown::HashMap;
use alloc::{string::String, vec::Vec, string::ToString};

#[derive(Debug, Clone, Default)]
pub struct Command<'a> {
    long: &'a str,
    short: &'a str,
    takes_input: bool,
    doc: &'a str,
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
pub struct Parser<'a> {
    input: &'a str,
    commands: Vec<Command<'a>>,
    doc_field: &'a str,
    name: &'a str,
    examples: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(name: &'a str, doc_field: &'a str, examples: &'a str) -> Self {
        Self {
            input: "",
            commands: Vec::new(),
            doc_field,
            name,
            examples
        }
    }

    pub fn add_command(&mut self, name: &'a str, takes_input: bool, short: &'a str, doc: &'a str) {
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

    pub fn parse(&mut self, input: &'a str) -> Result<HashMap<&'static str, &'static str>, &'a str> {
        self.input = input;

        let mut args: Vec<&str> = Vec::new();

        let mut in_quotes = false;
        let mut cur = "";
        for c in self.input.chars() {
            if c == '"' {
                in_quotes = !in_quotes;
            } else if c == ' ' && !in_quotes {
                if !cur.is_empty() {
                    args.push(cur.clone());
                    cur = "String::new()";
                }
            } else {
                cur = format_args!("{}{}", cur, c).as_str().unwrap();
            }
        }

        if !cur.is_empty() {
            args.push(cur);
        }


        let mut result: HashMap<&str, &'a str> = HashMap::new();
        let mut i = 0;

        while i < args.len() {
            let arg = args[i];

            if ["-h", "--help"].contains(&arg) {
                result.insert("help", "present");
            } else if arg.starts_with("--") {
                let (key, value) = Self::parse_long_arg(arg);

                if value.is_empty() {
                    let cmd = self.search(&key);
                    match cmd {
                        Some(command) => {
                            if command.takes_input {
                                if i + 1 >= args.len() {
                                    return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                                } else {
                                    let next_arg = &args[i + 1];
                                    if next_arg.starts_with("--") || next_arg.starts_with("-") {
                                        return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                                    }
                                    result.insert(key.clone(), next_arg.clone());
                                    i += 1;
                                }
                            } else {
                                result.insert(key.clone(), "present");
                            }
                        },
                        None => return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                    }
                } else {
                    if !self.check(&key) {
                        return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                    }
                    result.insert(key, value);
                }
            } else if arg.starts_with("-") {
                let (key, value) = self.parse_short_arg(arg);

                if value.is_empty() {
                    let cmd = self.search(&key);
                    match cmd {
                        Some(command) => {
                            if command.takes_input {
                                if i + 1 >= args.len() {
                                    return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                                } else {
                                    let next_arg = &args[i + 1];
                                    if next_arg.starts_with("--") || next_arg.starts_with("-") {
                                        return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                                    }
                                    result.insert(key.clone(), next_arg.clone());
                                    i += 1;
                                }
                            } else {
                                result.insert(key.clone(), "present");
                            }
                        },
                        None => return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                    }
                } else {
                    if !self.check(&key) {
                        return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                    }
                    result.insert(key, value);
                }
            } else {
                let flag = Self::parse_flag(arg);
                if !self.check(&flag) {
                    return Err(format_args!("Invalid argument: {}", arg).as_str().unwrap())
                }
                result.insert(flag, "present");
            }

            i += 1;
        }

        let mut out = "";

        if result.contains_key("help") {
            // Replace println! with your custom output function
            out = format_args!("Usage: {} [OPTIONS] ...\n\n{}\n", self.name, self.doc_field).as_str().unwrap();

            for command in self.commands.clone() {
                out = format_args!("{}\n  -{} --{}: {} ({})\n", out, command.short, command.long, command.doc, if command.takes_input { "takes input" } else { "flag" }).as_str().unwrap();
            }
            out = format_args!("{} Examples:\n", out).as_str().unwrap();
            for line in self.examples.lines() {
                out = format_args!("{}    {}\n",out, line).as_str().unwrap();
            }
            out = format_args!("{}\n", out).as_str().unwrap();
        }

        Ok(result)
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

    fn parse_short_arg(&self, arg: &str) -> (&str, &str) {
        let key = self.search(arg.get(1..=1).unwrap()).unwrap_or_default().long;
        let value = if arg.len() > 3 {
            &arg[3..]
        } else {
            ""
        };
        (key, value)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let mut tester = Parser::new("test", "A test program", "test -n=\"John Doe\" --age=20");
        tester.add_command("name", true, "n", "The name of the person");
        tester.add_command("age", true, "a", "The age of the person");
        let hash = tester.parse("-n=\"John Doe\" --age=20 -h");
        println!("{:?}", hash);
    }
}
