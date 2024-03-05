use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::{env, io};

struct SingleCommand {
    tokens: Vec<String>,
    output_filename: Option<String>,
    input_filename: Option<String>,
    directed_output: bool,
    directed_input: bool,
}

fn main() -> io::Result<()> {
    loop {
        let current_dir = match env::current_dir() {
            Ok(current_dir) => current_dir,
            Err(e) => {
                eprintln!("Failed to get current working directory: {}", e);
                return Err(e);
            }
        };
        print!("crush: {} > ", current_dir.display());
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

        let _ = execute_command(single_command);
    }
}

fn parse_user_input(tokens: Vec<&str>) -> SingleCommand {
    let mut command = SingleCommand {
        tokens: vec![],
        output_filename: None,
        input_filename: None,
        directed_output: false,
        directed_input: false,
    };

    let len = tokens.len();
    let mut i = 0;

    while i < len {
        if tokens[i] == ">" {
            command.directed_output = true;
            if (i + 1) < len && !is_operator(tokens[i + 1]) {
                command.output_filename = Some(tokens[i + 1].to_owned());
                i += 1;
            }
        } else if tokens[i] == "<" {
            command.directed_input = true;
            if (i + 1) < len && !is_operator(tokens[i + 1]) {
                command.input_filename = Some(tokens[i + 1].to_owned());
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
    if token == ">" || token == "<" {
        return true;
    }

    return false;
}

fn execute_command(command: SingleCommand) {
    let mut handles = vec![];

    let stdout_handle: Stdio = if command.directed_output {
        let output_file_path = command.output_filename.unwrap();
        let output_file = File::create(&output_file_path).unwrap();
        Stdio::from(output_file)
    } else {
        Stdio::inherit()
    };

    let stdin_handle: Stdio = if command.directed_input {
        let input_file_path = command.input_filename.unwrap();
        let input_file = File::open(&input_file_path).unwrap();
        Stdio::from(input_file)
    } else {
        Stdio::inherit()
    };

    let result = Command::new(command.tokens[0].as_str())
        .args(&command.tokens[1..])
        .stdin(Stdio::from(stdin_handle))
        .stdout(Stdio::from(stdout_handle))
        .spawn();

    match result {
        Ok(handle) => {
            handles.push(handle);
        }
        Err(err) => {
            eprintln!(
                "Failed to execute '{}': {}",
                command.tokens[0].as_str(),
                err
            );
        }
    };

    for mut handle in handles {
        if let Err(err) = handle.wait() {
            eprintln!("Failed to wait for command to finish: {}", err);
        }
    }
}
