use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    loop {
        print!("crush: > ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).expect("Failed to read input");

        let user_input = user_input.trim();

        if user_input == "exit" {
            return Ok(());
        }

        if user_input == ""{
            continue;
        }

        let mut tokens: Vec<&str> = user_input.split_whitespace().collect();
        let exe = tokens[0];
        tokens.remove(0);

        let mut handles = vec![];

        let command = Command::new(exe).args(tokens)
            .stdout(Stdio::inherit())
            .spawn();      

            match command {
                Ok(handle) => {
                    handles.push(handle); // Store the handle
                },
                Err(err) => {
                    eprintln!("Failed to execute '{}': {}", exe, err);
                    continue;
                }
            };
                    // Wait for all processes to finish
        for mut handle in handles {
            handle.wait()?;
        }
    }
}
