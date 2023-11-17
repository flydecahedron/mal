extern crate rustyline;

mod error;
mod printer;
mod reader;
mod types;

use error::Error;
use printer::print_value;
use reader::read;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as ReadlineResult};
use std::result::Result;
use types::Value;

fn eval(ast: &mut Value) -> Result<&Value, Error> {
    Ok(ast)
}

fn print(ast: &Value) -> Result<(), Error> {
    print_value(ast);
    Ok(())
}

fn read_eval_print(input: &str) -> Result<(), Error> {
    let mut ast = read(input).unwrap();
    eval(&mut ast)?;
    print(&ast)?;
    Ok(())
}
fn main() -> Result<(), Error> {
    let mut rl = DefaultEditor::new().unwrap();
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("user >");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                read_eval_print(&line)?;
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    Ok(())
}
