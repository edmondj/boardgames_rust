use boards::random_engine::DefaultRandomEngine;
use lazy_static::lazy_static;
use regex::Regex;
use solitaire_backend::*;
use std::io::Write;
use std::result::Result;

enum ParseActionError {
    Invalid(String),
}

fn main() {
    let mut rand = DefaultRandomEngine::new();
    let mut state = MemoryGame::new(&mut rand);

    loop {
        println!("{}", state);
        println!("[0] [1] [2] [3] [4] [5] [6]");
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() {
            break;
        }

        let line = line.trim();

        lazy_static! {
            static ref BUILD: Regex = Regex::new(r"build (\d+|u)").unwrap();
            static ref MOVE: Regex = Regex::new(r"move ((\d+) (\d+)|u) (\d+)").unwrap();
        }

        let action: Result<Action, ParseActionError> = if line == "draw" {
            Ok(Action::Draw)
        } else if let Some(cap) = BUILD.captures(&line) {
            Ok(Action::BuildFoundation {
                src: match cap.get(1).unwrap().as_str() {
                    "u" => FoundationSource::Upturned,
                    s => FoundationSource::Tableau(s.parse().unwrap()),
                },
            })
        } else if let Some(cap) = MOVE.captures(&line) {
            Ok(Action::BuildTableau {
                src: match cap.get(1).unwrap().as_str() {
                    "u" => TableauSource::Upturned,
                    _ => TableauSource::Tableau {
                        index: cap.get(2).unwrap().as_str().parse().unwrap(),
                        size: cap.get(3).unwrap().as_str().parse().unwrap(),
                    },
                },
                dst: cap.get(4).unwrap().as_str().parse().unwrap(),
            })
        } else if line == "quit" {
            break;
        } else {
            Err(ParseActionError::Invalid(format!(
                "Unknown command {}",
                line
            )))
        };

        use ActionResult::*;
        match action {
            Err(ParseActionError::Invalid(s)) => println!("{}", s),
            Ok(action) => match state.act(action) {
                Victory => {
                    println!("Congratulations! You won!");
                    break;
                }
                Failed(s) => println!("Invalid move: {}", s),
                OnGoing => (),
            },
        }
    }
}
