use boards::random_engine::DefaultRandomEngine;
use futures_core;
use solitaire_backend::{Card, Foundation, State, Suite, Tableau};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio;
use uuid::Uuid;

mod proto {
    pub mod solitaire {
        include!("proto/solitaire.rs");
    }
}

impl From<Suite> for proto::solitaire::Suite {
    fn from(src: Suite) -> Self {
        match src {
            Suite::Hearts => Self::Hearts,
            Suite::Diamonds => Self::Diamonds,
            Suite::Clubs => Self::Clubs,
            Suite::Spades => Self::Spades,
        }
    }
}

fn suite_to_proto(suite: proto::solitaire::Suite) -> i32 {
    suite.into()
}

impl From<&Card> for proto::solitaire::Card {
    fn from(src: &Card) -> Self {
        Self {
            suite: suite_to_proto(src.suite().into()),
            rank: src.rank() as u32,
        }
    }
}

impl From<Foundation> for proto::solitaire::Foundation {
    fn from(src: Foundation) -> Self {
        Self {
            suite: suite_to_proto(src.suite.into()),
            value: match src.value {
                0 => None,
                v => Some(v as u32),
            },
        }
    }
}

impl From<&Tableau> for proto::solitaire::Tableau {
    fn from(src: &Tableau) -> Self {
        Self {
            downfaced_len: src.downfaced_len() as u64,
            upturned: src.upturned_iter().map(|u| u.into()).collect(),
        }
    }
}

impl From<&State> for proto::solitaire::State {
    fn from(src: &State) -> Self {
        Self {
            draw_pile_size: src.draw_pile().len() as u32,
            upturned: src.upturned().map(|u| u.into()),
            foundations: src.foundations().iter().map(|f| f.into()).collect(),
            tableaus: src.tableaus().iter().map(|t| t.into()).collect(),
        }
    }
}

struct ActiveGame {
    state: State,
}

impl Default for ActiveGame {
    fn default() -> Self {
        Self {
            state: State::new(&mut DefaultRandomEngine::new()),
        }
    }
}

#[derive(Default)]
struct SolitareServiceState {
    games: HashMap<Uuid, ActiveGame>,
}

#[derive(Default)]
struct SolitaireService {
    state: Mutex<SolitareServiceState>,
}

struct SolitaireWatchStream;

impl futures_core::Stream for SolitaireWatchStream {
    type Item = Result<proto::solitaire::WatchResponse, tonic::Status>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::option::Option<<Self as futures_core::Stream>::Item>> {
        todo!()
    }
}

#[tonic::async_trait]
impl proto::solitaire::solitaire_server::Solitaire for SolitaireService {
    async fn create_game(
        &self,
        _request: tonic::Request<proto::solitaire::CreateGameRequest>,
    ) -> Result<tonic::Response<proto::solitaire::CreateGameResponse>, tonic::Status> {
        let id = Uuid::new_v4();
        let mut state = self.state.lock().unwrap();
        state.games.insert(id, ActiveGame::default());
        let ref game_state = state.games.get(&id).unwrap();
        Ok(
            tonic::Response::<proto::solitaire::CreateGameResponse>::new(
                proto::solitaire::CreateGameResponse {
                    id: id.to_string(),
                    state: Some((&game_state.state).into()),
                },
            ),
        )
    }

    async fn destroy_game(
        &self,
        request: tonic::Request<proto::solitaire::DestroyGameRequest>,
    ) -> Result<tonic::Response<proto::solitaire::DestroyGameResponse>, tonic::Status> {
        todo!();
    }

    async fn act(
        &self,
        request: tonic::Request<proto::solitaire::ActRequest>,
    ) -> Result<tonic::Response<proto::solitaire::ActResponse>, tonic::Status> {
        todo!();
    }

    type WatchStream = SolitaireWatchStream;

    async fn watch(
        &self,
        request: tonic::Request<proto::solitaire::WatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        todo!();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let service = SolitaireService::default();

    println!("Bookstore server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(proto::solitaire::solitaire_server::SolitaireServer::new(
            service,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
