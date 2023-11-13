extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn read(rl: &mut DefaultEditor) -> Result<String> {
    rl.readline("user > ")
}

fn eval(line: String) -> Result<String> {
    Ok(line)
}

fn print(s: String) -> Result<()> {
    println!("{}", s);
    Ok(())
}

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = read(&mut rl);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                print(eval(line)?)?;
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
