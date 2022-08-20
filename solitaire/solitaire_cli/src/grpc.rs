use async_trait::async_trait;
use solitaire_grpc::{proto::solitaire_client::SolitaireClient, ProtoError};
use std::fmt;

pub struct GrpcGame {
    client: SolitaireClient<tonic::transport::Channel>,
    id: String,
    state: solitaire_grpc::proto::State,
}

#[derive(Debug)]
pub enum NewGameError {
    ConnectError(tonic::transport::Error),
    CreateGameError(tonic::Status),
    NoState,
}

impl GrpcGame {
    pub async fn new(addr: String) -> Result<Self, NewGameError> {
        let mut client = SolitaireClient::connect(addr)
            .await
            .map_err(|e| NewGameError::ConnectError(e))?;
        let response = client
            .create_game(tonic::Request::new(
                solitaire_grpc::proto::CreateGameRequest {},
            ))
            .await
            .map_err(|e| NewGameError::CreateGameError(e))?
            .into_inner();
        Ok(Self {
            client,
            id: response.id,
            state: match response.state {
                None => Err(NewGameError::NoState),
                Some(state) => Ok(state),
            }?,
        })
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}

#[async_trait]
impl solitaire_backend::Game for GrpcGame {
    fn draw_pile_size(&self) -> usize {
        self.state.draw_pile_size as usize
    }

    fn upturned(&self) -> Option<solitaire_backend::Card> {
        self.state.upturned.as_ref().map(|c| c.try_into().unwrap())
    }

    fn foundations(&self) -> solitaire_backend::Foundations {
        let mut foundations = solitaire_backend::Foundations::default();
        for foundation in self.state.foundations.iter() {
            foundations[solitaire_grpc::proto::Suite::from_i32(foundation.suite)
                .ok_or_else(|| ProtoError::InvalidValue("foundation.suite".to_owned()))
                .unwrap()
                .try_into()
                .unwrap()] = foundation.value.unwrap_or(0) as u8;
        }
        foundations
    }

    fn tableaus<'a>(&'a self) -> Vec<solitaire_backend::Tableau> {
        self.state
            .tableaus
            .iter()
            .map(|t| solitaire_backend::Tableau {
                downfaced_len: t.downfaced_len as usize,
                upturned: t.upturned.iter().map(|c| c.try_into().unwrap()).collect(),
            })
            .collect()
    }

    async fn act(&mut self, action: solitaire_backend::Action) -> solitaire_backend::ActionResult {
        let response = self
            .client
            .act(tonic::Request::new(solitaire_grpc::proto::ActRequest {
                id: self.id.clone(),
                action: Some(action.into()),
            }))
            .await;
        match response {
            Err(e) => solitaire_backend::ActionResult::Failed(format!("{e}")),
            Ok(response) => {
                let response = response.into_inner();
                if let Some(state) = response.state {
                    self.state = state;
                }
                if response.victory {
                    solitaire_backend::ActionResult::Victory
                } else {
                    solitaire_backend::ActionResult::OnGoing
                }
            }
        }
    }
}

impl fmt::Display for GrpcGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        solitaire_backend::display(self, f)
    }
}
