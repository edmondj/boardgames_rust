use solitaire_backend::{
    Action, Card, Foundation, FoundationSource, Game, MemoryGame, Suite, Tableau,
};

pub mod proto {
    tonic::include_proto!("solitaire");
}

#[derive(Debug)]
pub enum ProtoError {
    InvalidValue(String),
}

impl From<Suite> for proto::Suite {
    fn from(src: Suite) -> Self {
        match src {
            Suite::Hearts => Self::Hearts,
            Suite::Diamonds => Self::Diamonds,
            Suite::Clubs => Self::Clubs,
            Suite::Spades => Self::Spades,
        }
    }
}

impl TryInto<Suite> for proto::Suite {
    type Error = ProtoError;

    fn try_into(self) -> Result<Suite, Self::Error> {
        match self {
            Self::Hearts => Ok(Suite::Hearts),
            Self::Diamonds => Ok(Suite::Diamonds),
            Self::Clubs => Ok(Suite::Clubs),
            Self::Spades => Ok(Suite::Spades),
            Self::Undefined => Err(ProtoError::InvalidValue(String::new())),
        }
    }
}

pub fn suite_to_proto(suite: proto::Suite) -> i32 {
    suite.into()
}

impl From<&Card> for proto::Card {
    fn from(src: &Card) -> Self {
        Self {
            suite: suite_to_proto(src.suite().into()),
            rank: src.rank() as u32,
        }
    }
}

impl TryInto<Card> for &proto::Card {
    type Error = ProtoError;

    fn try_into(self) -> Result<Card, Self::Error> {
        Ok(Card {
            suite: proto::Suite::from_i32(self.suite)
                .ok_or_else(|| ProtoError::InvalidValue("suite".to_owned()))?
                .try_into()?,
            rank: self.rank as u8,
        })
    }
}

impl From<Foundation> for proto::Foundation {
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

impl From<&Tableau> for proto::Tableau {
    fn from(src: &Tableau) -> Self {
        Self {
            downfaced_len: src.downfaced_len as u64,
            upturned: src.upturned.iter().map(|u| u.into()).collect(),
        }
    }
}

impl From<&MemoryGame> for proto::State {
    fn from(src: &MemoryGame) -> Self {
        Self {
            draw_pile_size: src.draw_pile_size() as u32,
            upturned: src.upturned().as_ref().map(|u| u.into()),
            foundations: src.foundations().iter().map(|f| f.into()).collect(),
            tableaus: src.tableaus().iter().map(|t| t.into()).collect(),
        }
    }
}

impl From<Action> for proto::Action {
    fn from(src: Action) -> Self {
        proto::Action {
            action: Some(match src {
                Action::Draw => proto::action::Action::Draw(proto::action::Draw {}),
                Action::BuildFoundation { src } => {
                    use proto::action::build_foundation;
                    proto::action::Action::BuildFoundation(proto::action::BuildFoundation {
                        source: Some(match src {
                            FoundationSource::Upturned => {
                                build_foundation::Source::Upturned(build_foundation::Upturned {})
                            }
                            FoundationSource::Tableau(index) => {
                                build_foundation::Source::Tableau(build_foundation::Tableau {
                                    index: index as u32,
                                })
                            }
                        }),
                    })
                }
                Action::BuildTableau { src, dst } => {
                    use proto::action::build_tableau;
                    proto::action::Action::BuildTableau(proto::action::BuildTableau {
                        source: Some(match src {
                            solitaire_backend::TableauSource::Upturned => {
                                build_tableau::Source::Upturned(build_tableau::Upturned {})
                            }
                            solitaire_backend::TableauSource::Tableau { index, size } => {
                                build_tableau::Source::Tableau(build_tableau::Tableau {
                                    index: index as u32,
                                    size: size as u32,
                                })
                            }
                        }),
                        destination_index: dst as u32,
                    })
                }
            }),
        }
    }
}

impl TryInto<Action> for &proto::Action {
    type Error = tonic::Status;
    fn try_into(self) -> Result<Action, Self::Error> {
        use proto::action::*;
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
