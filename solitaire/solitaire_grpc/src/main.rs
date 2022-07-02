use boards::random_engine::DefaultRandomEngine;
use solitaire_backend::{ActionResult, State};
use std::collections::HashMap;
use std::str::FromStr;
use tokio;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;
mod serial;
use serial::*;

type WatchMessage = Result<proto::solitaire::WatchResponse, tonic::Status>;

struct ActiveGame {
    state: State,
    streams: Vec<mpsc::Sender<WatchMessage>>,
}

impl Default for ActiveGame {
    fn default() -> Self {
        Self {
            state: State::new(&mut DefaultRandomEngine::new()),
            streams: Vec::default(),
        }
    }
}

impl ActiveGame {
    async fn send_watch_message(&mut self, ref msg: WatchMessage) {
        let mut i = 0usize;
        while i < self.streams.len() {
            if self.streams[i]
                .send(match msg {
                    Err(status) => Err(tonic::Status::new(status.code(), status.message())),
                    Ok(msg) => Ok(msg.clone()),
                })
                .await
                .is_ok()
            {
                i += 1;
            } else {
                self.streams.swap_remove(i);
            }
        }
        // self.streams.retain(move |stream| async {
        //     stream
        //         .send(match msg {
        //             Err(status) => Err(tonic::Status::new(status.code(), status.message())),
        //             Ok(msg) => Ok(msg.clone()),
        //         })
        //         .await
        //         .is_ok()
        // })
    }
}

#[derive(Default)]
struct SolitareServiceState {
    games: HashMap<Uuid, ActiveGame>,
}

fn new_not_found_status(id: &Uuid) -> tonic::Status {
    tonic::Status::not_found(format!("Game not found: {id}"))
}

impl SolitareServiceState {
    fn get_mut_game(&mut self, id: &Uuid) -> Result<&mut ActiveGame, tonic::Status> {
        match self.games.get_mut(&id) {
            None => Err(new_not_found_status(&id)),
            Some(game) => Ok(game),
        }
    }
}

#[derive(Default)]
struct SolitaireService {
    state: Mutex<SolitareServiceState>,
}

fn try_parse_id(id: &str) -> Result<Uuid, tonic::Status> {
    Uuid::from_str(id).map_err(|err| tonic::Status::invalid_argument(format!("Invalid id: {err}")))
}

#[tonic::async_trait]
impl proto::solitaire::solitaire_server::Solitaire for SolitaireService {
    async fn create_game(
        &self,
        _request: tonic::Request<proto::solitaire::CreateGameRequest>,
    ) -> Result<tonic::Response<proto::solitaire::CreateGameResponse>, tonic::Status> {
        let id = Uuid::new_v4();
        let mut state = self.state.lock().await;
        state.games.insert(id, ActiveGame::default());
        let ref game_state = state.games.get(&id).unwrap();
        Ok(tonic::Response::new(proto::solitaire::CreateGameResponse {
            id: id.to_string(),
            state: Some((&game_state.state).into()),
        }))
    }

    async fn destroy_game(
        &self,
        request: tonic::Request<proto::solitaire::DestroyGameRequest>,
    ) -> Result<tonic::Response<proto::solitaire::DestroyGameResponse>, tonic::Status> {
        let id = try_parse_id(&request.get_ref().id)?;
        let mut state = self.state.lock().await;
        match state.games.remove(&id) {
            None => Err(new_not_found_status(&id)),
            Some(mut game) => {
                std::mem::drop(state);
                game.send_watch_message(Err(tonic::Status::ok("Game destroyed")))
                    .await;
                Ok(tonic::Response::new(
                    proto::solitaire::DestroyGameResponse {},
                ))
            }
        }
    }

    async fn act(
        &self,
        request: tonic::Request<proto::solitaire::ActRequest>,
    ) -> Result<tonic::Response<proto::solitaire::ActResponse>, tonic::Status> {
        // let request = request;
        let id = try_parse_id(&request.get_ref().id)?;
        match request.into_inner().action {
            None => Err(tonic::Status::invalid_argument("Missing field 'action'")),
            Some(proto_action) => {
                let action = (&proto_action).try_into()?;
                let mut state = self.state.lock().await;
                let ref mut game = state.get_mut_game(&id)?;
                let result = game.state.act(action);
                if let ActionResult::Failed(s) = result {
                    Err(tonic::Status::failed_precondition(format!(
                        "Invalid move: {s}"
                    )))
                } else {
                    game.send_watch_message(Ok(proto::solitaire::WatchResponse {
                        action: Some(proto_action),
                        state: Some((&game.state).into()),
                    }))
                    .await;
                    let new_state = (&game.state).into();
                    std::mem::drop(state);
                    Ok(tonic::Response::new(proto::solitaire::ActResponse {
                        victory: match result {
                            ActionResult::Victory => true,
                            _ => false,
                        },
                        state: Some(new_state),
                    }))
                }
            }
        }
    }

    type WatchStream = ReceiverStream<WatchMessage>;

    async fn watch(
        &self,
        request: tonic::Request<proto::solitaire::WatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let id = try_parse_id(&request.get_ref().id)?;
        let mut state = self.state.lock().await;
        let ref mut game = state.get_mut_game(&id)?;
        let (tx, rx) = mpsc::channel(128);
        tx.send(Ok(proto::solitaire::WatchResponse {
            action: None,
            state: Some((&game.state).into()),
        }))
        .await
        .unwrap();
        game.streams.push(tx);
        Ok(tonic::Response::new(ReceiverStream::new(rx)))
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
