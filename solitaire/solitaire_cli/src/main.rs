use boards::random_engine::DefaultRandomEngine;
use solitaire_backend::*;
use std::io::Write;
use std::str::FromStr;
use std::{env, fmt};
mod grpc;
use grpc::{GrpcGame, NewGameError};

pub trait DisplayableGame: Game + fmt::Display {}

impl DisplayableGame for MemoryGame {}
impl DisplayableGame for GrpcGame {}

fn new_memory_game() -> Box<dyn DisplayableGame> {
    let mut rand = DefaultRandomEngine::new();
    Box::new(MemoryGame::new(&mut rand))
}

async fn new_grpc_game(addr: String) -> Result<Box<dyn DisplayableGame>, NewGameError> {
    GrpcGame::new(addr)
        .await
        .map(|g| -> Box<dyn DisplayableGame> {
            println!("Starting grpc game {}", g.id());
            Box::new(g)
        })
}

enum GameOption {
    Memory,
    Grpc(String),
}

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1);
    let mut game_option = None;
    while let Some(arg) = args.next() {
        if arg == "--grpc" {
            match args.next() {
                None => {
                    panic!("--grpc was given without an address");
                }
                Some(addr) => {
                    if game_option.is_some() {
                        panic!("--grpc was given more than once");
                    }
                    game_option = Some(GameOption::Grpc(addr));
                }
            }
        }
    }

    let game_option = game_option.unwrap_or(GameOption::Memory);

    let mut game = match game_option {
        GameOption::Memory => new_memory_game(),
        GameOption::Grpc(addr) => match new_grpc_game(addr).await {
            Ok(game) => game,
            Err(e) => panic!("Failed to create grpc game: {:?}", e),
        },
    };

    loop {
        println!("{}", game);
        println!("[0] [1] [2] [3] [4] [5] [6]");
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() {
            break;
        }

        let line = line.trim();

        match Action::from_str(line) {
            Err(ParseActionError::Invalid(s)) => {
                if line == "quit" {
                    break;
                }
                println!("{}", s)
            }
            Ok(action) => match game.act(action).await {
                ActionResult::Victory => {
                    println!("Congratulations! You won!");
                    break;
                }
                ActionResult::Failed(s) => println!("Invalid move: {}", s),
                ActionResult::OnGoing => (),
            },
        }
    }
}
