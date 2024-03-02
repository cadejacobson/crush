use std::io;
use std::io::Write;

fn main() {
    loop {
        print!("crush: > ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).expect("Failed to read input");

        let user_input = user_input.trim();

        if user_input == "exit" {
            break;
        }

        if user_input == ""{
            continue;
        }
        println!("You entered: {}", user_input);
    }
}