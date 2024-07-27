mod rpn;
use rpn::{Machine, Token};

use clap::{Arg, Command, Subcommand};
use std::fs::File;
use std::io::{self, Read, Write}; // Import Write trait for flush

#[derive(Subcommand)]
enum Commands {
    Repl,
    Build,
    Rpn {
        #[arg(short, long)]
        input: String,
    },
}

fn main() {
    let matches = Command::new("calc_tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("repl")
                .about("Enter the REPL mode to evaluate expressions interactively."),
        )
        .subcommand(Command::new("build").about("Build the project (not yet implemented)."))
        .subcommand(
            Command::new("rpn")
                .about("Convert infix, postfix, or prefix expressions to reverse polish notation.")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("STRING or FILE")
                        .help("The expression string or path to a file containing the expression.")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("run").about("Evaluates expression").arg(
                Arg::new("input")
                    .short('i')
                    .long("input")
                    .value_name("STRING or FILE")
                    .help("The expression string or path to a file containing the expression.")
                    .required(true),
            ),
        )
        .get_matches();

    let mut machine = Machine::new();

    match matches.subcommand() {
        Some(("repl", _)) => {
            repl_mode(machine);
        }
        Some(("build", _)) => {
            println!("Build command is not yet implemented.");
        }
        Some(("rpn", sub_m)) => {
            let input = sub_m.get_one::<String>("input").unwrap();
            match read_input(input) {
                Ok(content) => println!("{}", machine.to_rpn(&content)),
                Err(e) => eprintln!("Error reading input: {}", e),
            }
        }
        Some(("run", sub_m)) => {
            let input = sub_m.get_one::<String>("input").unwrap();
            match read_input(input) {
                Ok(content) => print_result(machine.run(&content)),
                Err(e) => eprintln!("Error reading input: {}", e),
            }
        }
        _ => unreachable!(),
    }
}
fn print_result(result: Result<Token, String>) {
    match result.unwrap() {
        Token::Number(num) => println!("{} ", num),
        Token::String(s) => println!("\"{}\" ", s),
        Token::Variable(var) => println!("{} ", var),
        Token::Operator(op, _) => println!("{} ", op),
        _ => {}
    }
}

fn repl_mode(mut machine: Machine) {
    println!("Enter REPL mode. Type 'exit' to leave.");
    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush().unwrap(); // Flush stdout to ensure prompt is printed
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();
        if trimmed == "exit" {
            break;
        }
        let result = machine.run(trimmed);
        print_result(result);
    }
}

fn read_input(input: &str) -> io::Result<String> {
    if std::path::Path::new(input).exists() {
        let mut file = File::open(input)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    } else {
        Ok(input.to_string())
    }
}
