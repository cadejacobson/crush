use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

struct SingleCommand {
    tokens: Vec<String>,
    output_filename: Option<String>,
    directed_output: bool,
}

fn main() -> io::Result<()> {
    loop {
        print!("crush: > ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read input");

        let user_input = user_input.trim();

        if user_input == "exit" {
            return Ok(());
        }

        if user_input == "" {
            continue;
        }

        let tokens: Vec<&str> = user_input.split_whitespace().collect();
        let single_command = parse_user_input(tokens.clone());

        let mut handles = vec![];

        let command = Command::new(single_command.tokens[0].as_str())
            .args(&single_command.tokens[1..])
            .stdout(Stdio::inherit())
            .spawn();

        match command {
            Ok(handle) => {
                handles.push(handle);
            }
            Err(err) => {
                eprintln!(
                    "Failed to execute '{}': {}",
                    single_command.tokens[0].as_str(),
                    err
                );
                continue;
            }
        };

        for mut handle in handles {
            handle.wait()?;
        }
    }
}

fn parse_user_input(tokens: Vec<&str>) -> SingleCommand {
    let mut command = SingleCommand {
        tokens: vec![],
        output_filename: None,
        directed_output: false,
    };

    let len = tokens.len();
    let mut i = 0;

    while i < len {
        if tokens[i] == ">" {
            command.directed_output = true;
            if i + 1 < len && !is_operator(tokens[i + 1]) {
                command.output_filename = Some(tokens[i + 1].to_owned());
                i += 1;
            }
        } else {
            command.tokens.push(tokens[i].to_owned());
        }
        i += 1;
    }

    return command;
}

fn is_operator(token: &str) -> bool {
    if token == ">" {
        return true;
    }

    return false;
}
