use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::fs::File;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::{env, io};

#[derive(Clone)]
struct SingleCommand {
    tokens: Vec<String>,
    output_filename: Option<String>,
    input_filename: Option<String>,
    piped_input: bool,     /* For the pipe operator | */
    piped_output: bool,    /* For the pipe operator | */
    directed_output: bool, /* For the file operator > */
    directed_input: bool,  /* For the file operator > */
}

fn main() -> io::Result<()> {
    let mut user_input_history: Vec<String> = Vec::new();
    let mut input_history_index: usize = 0;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(event) = event::read().unwrap() {
                tx.send(event).unwrap();
            }
        }
    });

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

        enable_raw_mode().expect("Failed to enable raw mode");
        loop {
            if let Ok(event) = rx.recv() {
                match event.code {
                    KeyCode::Enter => {
                        print!("\n\r\x1B[K");
                        io::stdout().flush().expect("Failed to flush stdout");
                        break;
                    }
                    KeyCode::Backspace => {
                        user_input.pop();
                        print!("\r\x1B[K");
                        print!("crush: {} > {}", current_dir.display(), user_input);
                        io::stdout().flush().expect("Failed to flush stdout");
                    }
                    KeyCode::Char(c) => {
                        user_input.push(c);
                        print!("\r\x1B[K");
                        print!("crush: {} > {}", current_dir.display(), user_input);
                        io::stdout().flush().expect("Failed to flush stdout");
                    }
                    KeyCode::Up => {
                        if input_history_index > 0 {
                            input_history_index = input_history_index - 1;
                            user_input = user_input_history[input_history_index].to_owned();
                            print!("\r\x1B[K");
                            print!("crush: {} > {}", current_dir.display(), user_input);
                            io::stdout().flush().expect("Failed to flush stdout");
                        }
                    }
                    KeyCode::Down => {
                        if input_history_index < user_input_history.len() - 1 {
                            input_history_index = input_history_index + 1;
                            user_input = user_input_history[input_history_index].to_owned();
                            print!("\r\x1B[K");
                            print!("crush: {} > {}", current_dir.display(), user_input);
                            io::stdout().flush().expect("Failed to flush stdout");
                        }
                    }

                    _ => {}
                }
            }
        }
        disable_raw_mode().expect("Failed to disable raw mode");


        let user_input = user_input.trim();

        if user_input == "" {
            continue;
        }

        if user_input == "exit" {
            return Ok(());
        }

        let tokens: Vec<&str> = user_input.split_whitespace().collect();
        let commands = parse_user_input(tokens.clone());

        if commands[0].tokens[0] == "cd" {
            change_dir(commands[0].clone());
            continue;
        }

        let _ = execute_commands(commands);

        user_input_history.push(user_input.to_owned());
        input_history_index = user_input_history.len();
    }
}

fn parse_user_input(tokens: Vec<&str>) -> Vec<SingleCommand> {
    let mut commands = Vec::new();

    let len = tokens.len();
    let mut i = 0;
    let mut pipe_next_input = false;

    while i < len {
        let mut command = SingleCommand {
            tokens: vec![],
            output_filename: None,
            input_filename: None,
            piped_input: false,
            piped_output: false,
            directed_output: false,
            directed_input: false,
        };

        if pipe_next_input {
            command.piped_input = true;
        }

        pipe_next_input = false;

        while i < len && tokens[i] != "|" {
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

        if i != len {
            if tokens[i] == "|" {
                command.piped_output = true;
                pipe_next_input = true;
                i += 1;
            }
        }

        commands.push(command)
    }

    return commands;
}

fn is_operator(token: &str) -> bool {
    if token == ">" || token == "<" || token == "|" {
        return true;
    }

    return false;
}

fn execute_commands(commands: Vec<SingleCommand>) {
    let mut handles: Vec<Child> = vec![];

    for i in 0..commands.len() {
        let stdin_handle: Stdio = if commands[i].directed_input {
            let input_file_path = commands[i].input_filename.as_ref().unwrap();
            let input_file = File::open(&input_file_path).unwrap();
            Stdio::from(input_file)
        } else if commands[i].piped_input {
            Stdio::from(handles[i - 1].stdout.take().unwrap())
        } else {
            Stdio::inherit()
        };

        let stdout_handle: Stdio = if commands[i].directed_output {
            let output_file_path = commands[i].output_filename.as_ref().unwrap();
            let output_file = File::create(&output_file_path).unwrap();
            Stdio::from(output_file)
        } else if commands[i].piped_output {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };

        let result = Command::new(commands[i].tokens[0].as_str())
            .args(&commands[i].tokens[1..])
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
                    commands[i].tokens[0].as_str(),
                    err
                );
            }
        };
    }

    for mut handle in handles {
        if let Err(err) = handle.wait() {
            eprintln!("Failed to wait for command to finish: {}", err);
        }
    }
}

fn change_dir(command: SingleCommand) {
    if command.tokens.len() != 2 {
        println!("Incorrect amount of arguments for this function");
        return;
    }

    if let Err(e) = env::set_current_dir(command.tokens[1].as_str()) {
        eprintln!("Error: {}", e);
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::fs;

    #[test]
    fn test_piped_parse_user_input() {
        let user_input_test: &str = "ls | grep test";

        let tokens: Vec<&str> = user_input_test.split_whitespace().collect();
        let commands = parse_user_input(tokens);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].tokens, vec!["ls"]);
        assert_eq!(commands[0].piped_output, true);
        assert_eq!(commands[0].piped_input, false);
        assert_eq!(commands[1].tokens, vec!["grep", "test"]);
        assert_eq!(commands[1].piped_output, false);
        assert_eq!(commands[1].piped_input, true);
    }

    #[test]
    fn test_directed_output_parse_user_input(){
        let user_input_test: &str = "ls -l > ls.txt";

        let tokens: Vec<&str> = user_input_test.split_whitespace().collect();
        let commands = parse_user_input(tokens);
        
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].tokens, vec!["ls", "-l"]);
        assert_eq!(commands[0].directed_input, false);
        assert_eq!(commands[0].input_filename, None);
        assert_eq!(commands[0].directed_output, true);
        assert_eq!(commands[0].output_filename, Some("ls.txt".to_owned()));
        assert_eq!(commands[0].piped_output, false);
        assert_eq!(commands[0].piped_input, false);
    }

    #[test]
    fn test_directed_input_parse_user_input(){
        let user_input_test: &str = "sort < unsorted.txt";

        let tokens: Vec<&str> = user_input_test.split_whitespace().collect();
        let commands = parse_user_input(tokens);
        
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].tokens, vec!["sort"]);
        assert_eq!(commands[0].directed_input, true);
        assert_eq!(commands[0].input_filename, Some("unsorted.txt".to_owned()));
        assert_eq!(commands[0].directed_output, false);
        assert_eq!(commands[0].output_filename, None);
        assert_eq!(commands[0].piped_output, false);
        assert_eq!(commands[0].piped_input, false);
    }

    #[test]
    fn test_writing_to_file(){
        let user_input_test: &str = "sort < ./test/unsorted_skyrim.test > ./test/sorted_in_code.test";

        let tokens: Vec<&str> = user_input_test.split_whitespace().collect();
        let commands = parse_user_input(tokens);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].tokens, vec!["sort"]);
        assert_eq!(commands[0].directed_input, true);
        assert_eq!(commands[0].input_filename, Some("./test/unsorted_skyrim.test".to_owned()));
        assert_eq!(commands[0].directed_output, true);
        assert_eq!(commands[0].output_filename, Some("./test/sorted_in_code.test".to_owned()));
        assert_eq!(commands[0].piped_output, false);
        assert_eq!(commands[0].piped_input, false);

        execute_commands(commands);

        let unsorted_content = fs::read_to_string("./test/sorted_skyrim.test")
            .expect("Unable to read sorted file");
        
        let sorted_content = fs::read_to_string("./test/sorted_in_code.test")
            .expect("Unable to read sorted file");

        assert_eq!(unsorted_content, sorted_content);

        fs::remove_file("./test/sorted_in_code.test").expect("Unable to delete sorted file");
        assert!(fs::metadata("./test/sorted_in_code.test").is_err(), "Sorted file should have been deleted");
    }
}