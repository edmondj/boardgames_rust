use solitaire_backend::{Action, Card, Foundation, Game, MemoryGame, Suite, Tableau};

pub mod proto {
    pub mod solitaire {
        tonic::include_proto!("solitaire");
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
            downfaced_len: src.downfaced_len as u64,
            upturned: src.upturned.iter().map(|u| u.into()).collect(),
        }
    }
}

impl From<&MemoryGame> for proto::solitaire::State {
    fn from(src: &MemoryGame) -> Self {
        Self {
            draw_pile_size: src.draw_pile_size() as u32,
            upturned: src.upturned().as_ref().map(|u| u.into()),
            foundations: src.foundations().iter().map(|f| f.into()).collect(),
            tableaus: src.tableaus().iter().map(|t| t.into()).collect(),
        }
    }
}

impl TryInto<Action> for &proto::solitaire::Action {
    type Error = tonic::Status;
    fn try_into(self) -> Result<Action, Self::Error> {
        use proto::solitaire::action::*;
        match self
            .action
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Missing field `action`"))?
        {
            Action::Draw(_) => Ok(solitaire_backend::Action::Draw),
            Action::BuildFoundation(f) => {
                use build_foundation::*;
                Ok(solitaire_backend::Action::BuildFoundation {
                    src: match f.source.as_ref().ok_or_else(|| {
                        tonic::Status::invalid_argument(
                            "Missing field `action.build_foundation.source`",
                        )
                    })? {
                        Source::Upturned(_) => solitaire_backend::FoundationSource::Upturned,
                        Source::Tableau(t) => {
                            solitaire_backend::FoundationSource::Tableau(t.index as usize)
                        }
                    },
                })
            }
            Action::BuildTableau(t) => {
                use build_tableau::*;
                Ok(solitaire_backend::Action::BuildTableau {
                    src: match t.source.as_ref().ok_or_else(|| {
                        tonic::Status::invalid_argument(
                            "Missing field `action.build_tableau.source`",
                        )
                    })? {
                        Source::Upturned(_) => solitaire_backend::TableauSource::Upturned,
                        Source::Tableau(t) => solitaire_backend::TableauSource::Tableau {
                            index: t.index as usize,
                            size: t.size as usize,
                        },
                    },
                    dst: t.destination_index as usize,
                })
            }
        }
    }
}
