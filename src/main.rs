use anyhow::Result;
use clap::Parser;
use crossterm::style::Stylize;
use journal::cli::{Cli, Command};
use journal::handler::Handler;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Command::Repl = cli.command {
        return repl();
    }

    let handler = Handler::new()?;
    if let Err(err) = handler.handle(cli) {
        eprintln!("{}: {}", "error".red(), err);
    }

    Ok(())
}

fn repl() -> Result<()> {
    let handler = Handler::new()?;
    let mut rl = DefaultEditor::new()?;

    // #[cfg(feature = "with-file-history")]
    // if rl.load_history("history.txt").is_err() {
    //     println!("No previous history.");
    // }

    println!("{}", JOURNAL);

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let mut line: Vec<&str> = line.split_whitespace().collect();
                line.insert(0, "jn");

                match Cli::try_parse_from(&line) {
                    Ok(cli) => {
                        if let Err(err) = handler.handle(cli) {
                            eprintln!("{}: {}", "error".red(), err);
                        }
                        println!();
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
            Err(err) => match err {
                ReadlineError::Interrupted | ReadlineError::Eof => break,
                err => {
                    eprintln!("{}: {}", "error".red(), err);
                    break;
                }
            },
        }
    }

    // #[cfg(feature = "with-file-history")]
    // rl.save_history("history.txt");

    Ok(())
}

const JOURNAL: &str = r"
      _                              _ 
     | | ___  _   _ _ __ _ __   __ _| |
  _  | |/ _ \| | | | '__| '_ \ / _` | |
 | |_| | (_) | |_| | |  | | | | (_| | |
  \___/ \___/ \__,_|_|  |_| |_|\__,_|_|
                                       

Welcome to journal REPL.

Press ctrl-c or ctrl-d to exit.
";
