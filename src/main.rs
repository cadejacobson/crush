use std::io;
use std::io::Write;
use std::process::{Command, Stdio};
use std::os::fd::IntoRawFd;
use std::fs::File;
use std::path::{Path, PathBuf};

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

        let _ = execute_command(single_command);
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
            if (i + 1) < len && !is_operator(tokens[i + 1]) {
                command.output_filename = Some(tokens[i + 1].to_owned());
                i += 1;
            }
        } else {
            command.tokens.push(tokens[i].to_owned());
        }
        i += 1;
    }

    println!("{}", command.directed_output);
    println!("{:?}", command.output_filename);

    return command;
}

fn is_operator(token: &str) -> bool {
    if token == ">" {
        return true;
    }

    return false;
}

fn execute_command(command: SingleCommand){
    let mut handles = vec![];
    
    let stdout_handle: Stdio = if command.directed_output {
        let file_path = command.output_filename.unwrap(); 
        let file = File::create(&file_path).unwrap();
        Stdio::from(file)
    } else {
        Stdio::inherit()
    };


    let result = Command::new(command.tokens[0].as_str())
        .args(&command.tokens[1..])
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