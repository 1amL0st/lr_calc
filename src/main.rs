use std::io::Write;

mod ast;
mod evaluator;
mod scanner;
use evaluator::evaluate;

fn main() {
    loop {
        print!(">>> ");
        std::io::stdout().flush().expect("Reading error!");

        let mut exp = String::new();
        match std::io::stdin().read_line(&mut exp) {
            Ok(_) => {
                if exp.starts_with("q") || exp.starts_with("exit") {
                    break;
                }
                match evaluate(&exp) {
                    Ok(result) => println!("<<< {}", result),
                    Err(err) => println!("Error happened: {}", err),
                }
            }
            Err(_) => println!("Input reading error!"),
        }

        println!("exp = {}", exp);
    }
}
