#![no_std]

extern crate alloc;

use hashbrown::HashMap;
use alloc::{string::String, vec::Vec, string::ToString};

#[derive(Debug, Clone, Default)]
pub struct Command {
    long: String,
    short: char,
    takes_input: bool,
    doc: &'static str,
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
    doc: &'static str,
    name: &'static str,
    examples: &'static str,
}

impl Parser {
    pub fn new(name: &'static str, doc: &'static str, examples: &'static str) -> Self {
        Self {
            input: String::new(),
            commands: Vec::new(),
            doc,
            name,
            examples
        }
    }

    pub fn add_command(&mut self, name: String, takes_input: bool, short: char, doc: &'static str) {
        self.commands.push(Command {
            long: name,
            short,
            takes_input,
            doc,
        });
    }

    pub fn parse(&mut self, input: String) -> Result<HashMap<String, String>, String> {
        self.input = input;
    
        let mut args: Vec<String> = Vec::new();

        let mut in_quotes = false;
        let mut cur = String::new();
        for c in self.input.chars() {
            if c == '"' {
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

        if !cur.is_empty() {
            args.push(cur);
        }

        println!("{:?}", args);

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
                                    return Err(format!("Invalid argument: {}", arg))
                                } else {
                                    let next_arg = &args[i + 1];
                                    if next_arg.starts_with("--") || next_arg.starts_with("-") {
                                        return Err(format!("Invalid argument: {}", arg))
                                    }
                                    result.insert(key.clone(), next_arg.clone());
                                    i += 1;
                                }
                            } else {
                                result.insert(key.clone(), "present".to_string());
                            }
                        },
                        None => return Err(format!("Invalid argument: {}", arg))
                    }
                } else {
                    if !self.check(&key) {
                        return Err(format!("Invalid argument: {}", arg))
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
                                    return Err(format!("Invalid argument: {}", arg))
                                } else {
                                    let next_arg = &args[i + 1];
                                    if next_arg.starts_with("--") || next_arg.starts_with("-") {
                                        return Err(format!("Invalid argument: {}", arg))
                                    }
                                    result.insert(key.clone(), next_arg.clone());
                                    i += 1;
                                }
                            } else {
                                result.insert(key.clone(), "present".to_string());
                            }
                        },
                        None => return Err(format!("Invalid argument: {}", arg))
                    }
                } else {
                    if !self.check(&key) {
                        return Err(format!("Invalid argument: {}", arg))
                    }
                    result.insert(key, value);
                }
            } else {
                let flag = Self::parse_flag(arg);
                if !self.check(&flag) {
                    return Err(format!("Invalid argument: {}", arg))
                }
                result.insert(flag, "present".to_string());
            }

            i += 1;
        }

        if result.contains_key("help") {
            println!("Usage: {} [OPTIONS] ...\n\n{}\n", self.name, self.doc);
            for command in self.commands.clone() {
                println!("  -{} --{}: {} ({})", command.short, command.long, command.doc, if command.takes_input { "takes input" } else { "flag" });
            }
            println!("Examples:");
            for line in self.examples.lines() {
                println!("  {}", line);
            }

            println!("")
        }

        Ok(result)
    }

    fn search(&self, arg: &str) -> Option<Command> {
        for command in &self.commands {
            if arg == command.long || arg == command.short.to_string() {
                return Some(command.clone());
            }
        }
        None
    }

    fn parse_flag(arg: &str) -> String {
        let key = if arg.starts_with("--") {
            arg[2..].to_string()
        } else {
            arg[1..].to_string()
        };
        key
    }

    fn check(&self, arg: &str) -> bool {
        for command in &self.commands {
            if arg == command.long || arg == command.short.to_string() {
                return true;
            }
        }
        false
    }

    fn parse_long_arg(arg: &str) -> (String, String) {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        let key = parts[0][2..].to_string();
        let value = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            "".to_string()
        };
        (key, value)
    }

    fn parse_short_arg(&self, arg: &str) -> (String, String) {
        let key = self.search(arg.chars().nth(1).unwrap_or_default().to_string().as_str()).unwrap_or_default().long;
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

    #[test]
    fn test_parse() {
        let mut tester = Parser::new("test", "A test program", "test -n=\"John Doe\" --age=20");
        tester.add_command("name".to_string(), true, 'n', "The name of the person");
        tester.add_command("age".to_string(), true, 'a', "The age of the person");
        let hash = tester.parse("-n=\"John Doe\" --age=20 -h".to_string());
        println!("{:?}", hash);
    }
}
