use async_trait::async_trait;
use boards::cards::french::{standard_52_deck, KING};
pub use boards::cards::french::{Card, Suite};
use boards::cards::FrenchDeck;
use boards::random_engine::RandomEngine;
use core::fmt;
use lazy_static::lazy_static;
use regex::Regex;
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

#[derive(Debug, Clone, Default)]
struct MemoryTableau {
    pile: Vec<Card>,
    upturned: usize,
}

impl MemoryTableau {
    fn len(&self) -> usize {
        self.pile.len()
    }

    fn upturned_len(&self) -> usize {
        self.upturned
    }

    fn downfaced_len(&self) -> usize {
        self.len() - self.upturned_len()
    }

    fn upturned(&self, i: usize) -> Option<&Card> {
        if i < self.upturned {
            Some(&self.pile[self.downfaced_len() + i])
        } else {
            None
        }
    }

    fn upturned_iter(&self) -> impl Iterator<Item = &Card> {
        self.pile.iter().skip(self.downfaced_len())
    }

    fn bottom(&self) -> Option<&Card> {
        if self.upturned > 0 {
            self.pile.last()
        } else {
            None
        }
    }

    fn maybe_upturn(&mut self) {
        if self.upturned == 0 && self.pile.len() > 0 {
            self.upturned = 1; // Reveal new top card
        }
    }

    fn take_upturned(&mut self, size: usize) -> Vec<Card> {
        if size > self.upturned {
            panic!("Taking too much upturned cards");
        }
        self.upturned -= size;
        let sent = self.pile.drain((self.pile.len() - size)..).collect();
        self.maybe_upturn();
        sent
    }

    fn add_upturned(&mut self, cards: impl Iterator<Item = Card>) {
        for c in cards {
            self.pile.push(c);
            self.upturned += 1;
        }
    }

    fn remove_bottom(&mut self) {
        self.pile.pop();
        self.upturned = self.upturned.saturating_sub(1);
        self.maybe_upturn();
    }
}

pub struct Tableau {
    pub downfaced_len: usize,
    pub upturned: Vec<Card>,
}

#[derive(Default, Copy, Clone)]
pub struct Foundations {
    pub foundations: [u8; 4],
}

#[derive(Copy, Clone)]
pub struct Foundation {
    pub suite: Suite,
    pub value: u8,
}

impl Foundations {
    pub fn iter(&self) -> impl Iterator<Item = Foundation> {
        self.foundations.into_iter().enumerate().map(|(i, value)| {
            use Suite::*;
            Foundation {
                suite: match i {
                    0 => Hearts,
                    1 => Diamonds,
                    2 => Clubs,
                    3 => Spades,
                    _ => panic!("This should not happend"),
                },
                value,
            }
        })
    }
}

impl Index<Suite> for Foundations {
    type Output = u8;

    fn index(&self, index: Suite) -> &Self::Output {
        use Suite::*;
        match index {
            Hearts => &self.foundations[0],
            Diamonds => &self.foundations[1],
            Clubs => &self.foundations[2],
            Spades => &self.foundations[3],
        }
    }
}

impl IndexMut<Suite> for Foundations {
    fn index_mut(&mut self, index: Suite) -> &mut Self::Output {
        use Suite::*;
        match index {
            Hearts => &mut self.foundations[0],
            Diamonds => &mut self.foundations[1],
            Clubs => &mut self.foundations[2],
            Spades => &mut self.foundations[3],
        }
    }
}

pub const TABLEAUS_COUNT: usize = 7;

#[async_trait]
pub trait Game {
    fn draw_pile_size(&self) -> usize;
    fn upturned(&self) -> Option<Card>;
    fn foundations(&self) -> Foundations;
    fn tableaus<'a>(&'a self) -> Vec<Tableau>;

    async fn act(&mut self, action: Action) -> ActionResult;
}

#[derive(Clone)]
pub struct MemoryGame {
    draw_pile: FrenchDeck,
    waste: FrenchDeck,
    foundations: Foundations,
    tableaus: [MemoryTableau; TABLEAUS_COUNT],
}

#[async_trait]
impl Game for MemoryGame {
    fn draw_pile_size(&self) -> usize {
        self.draw_pile.len()
    }

    fn upturned(&self) -> Option<Card> {
        self.waste.peek().map(|c| *c)
    }

    fn foundations(&self) -> Foundations {
        self.foundations
    }

    fn tableaus<'a>(&'a self) -> Vec<Tableau> {
        self.tableaus
            .iter()
            .map(|t| Tableau {
                downfaced_len: t.downfaced_len(),
                upturned: t.upturned_iter().map(|&c| c).collect(),
            })
            .collect()
    }

    async fn act(&mut self, action: Action) -> ActionResult {
        use Action::*;
        use ActionResult::*;
        match action {
            Draw => {
                if self.draw_pile.is_empty() {
                    self.draw_pile.put_bottom_many(self.waste.draw_all().rev());
                }
                if let Some(c) = self.draw_pile.draw() {
                    self.waste.put_top(c);
                }
                OnGoing
            }
            BuildFoundation { src } => {
                use FoundationSource::*;
                let maybe_card = match src {
                    Upturned => self.waste.peek(),
                    Tableau(idx) => {
                        if idx >= self.tableaus.len() {
                            None
                        } else {
                            self.tableaus[idx].pile.last()
                        }
                    }
                };
                if let Some(c) = maybe_card {
                    let dest: &mut u8 = &mut self.foundations[c.suite()];
                    if c.rank() != *dest + 1 {
                        Failed(String::from("Invalid rank"))
                    } else {
                        *dest = c.rank();
                        std::mem::drop(maybe_card);
                        match src {
                            Tableau(idx) => self.tableaus[idx].remove_bottom(),
                            Upturned => {
                                self.waste.draw();
                            }
                        };
                        if self.foundations.iter().all(|f| f.value == KING) {
                            Victory
                        } else {
                            OnGoing
                        }
                    }
                } else {
                    Failed(String::from("No source card"))
                }
            }
            BuildTableau { src, dst } => {
                use TableauSource::*;
                let joint = match src {
                    Upturned => self.waste.peek(),
                    Tableau { index, size } => {
                        if index >= self.tableaus.len()
                            || size > self.tableaus[index].upturned_len()
                            || index == dst
                        {
                            None
                        } else {
                            let tableau = &self.tableaus[index];
                            tableau.upturned(size - tableau.upturned_len())
                        }
                    }
                };
                if dst >= self.tableaus.len() {
                    Failed(String::from("Invalid dest"))
                } else {
                    let dst_tableau = &self.tableaus[dst];
                    if let Some(joint) = joint {
                        let valid = match dst_tableau.bottom() {
                            None => joint.rank() == KING,
                            Some(bottom) => {
                                joint.suite().color() != bottom.suite().color()
                                    && joint.rank() == bottom.rank() - 1
                            }
                        };
                        if valid {
                            let cards = match src {
                                Upturned => vec![self.waste.draw().unwrap()],
                                Tableau { index, size } => self.tableaus[index].take_upturned(size),
                            };
                            self.tableaus[dst].add_upturned(cards.into_iter());
                            OnGoing
                        } else {
                            Failed(String::from("Invalid target"))
                        }
                    } else {
                        Failed(String::from("Invalid source"))
                    }
                }
            }
        }
    }
}

pub enum FoundationSource {
    Upturned,
    Tableau(usize),
}

pub enum TableauSource {
    Upturned,
    Tableau { index: usize, size: usize },
}

pub enum Action {
    Draw,
    BuildFoundation { src: FoundationSource },
    BuildTableau { src: TableauSource, dst: usize },
}

pub enum ParseActionError {
    Invalid(String),
}

impl FromStr for Action {
    type Err = ParseActionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref BUILD: Regex = Regex::new(r"build (\d+|u)").unwrap();
            static ref MOVE: Regex = Regex::new(r"move ((\d+) (\d+)|u) (\d+)").unwrap();
        }

        if s == "draw" {
            Ok(Action::Draw)
        } else if let Some(cap) = BUILD.captures(&s) {
            Ok(Action::BuildFoundation {
                src: match cap.get(1).unwrap().as_str() {
                    "u" => FoundationSource::Upturned,
                    s => FoundationSource::Tableau(s.parse().unwrap()),
                },
            })
        } else if let Some(cap) = MOVE.captures(&s) {
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
        } else {
            Err(ParseActionError::Invalid(format!("Unknown command {}", s)))
        }
    }
}

pub enum ActionResult {
    Victory,
    OnGoing,
    Failed(String),
}

impl MemoryGame {
    pub fn new(rand: &mut impl RandomEngine) -> Self {
        let mut draw_pile = standard_52_deck();
        FrenchDeck::shuffle(&mut draw_pile, rand);

        let tableaus = {
            let mut arr: [MaybeUninit<MemoryTableau>; TABLEAUS_COUNT] =
                unsafe { MaybeUninit::uninit().assume_init() };
            for i in 0..arr.len() {
                arr[i].write(MemoryTableau {
                    pile: draw_pile.draw_many(i + 1).collect(),
                    upturned: 1,
                });
            }
            unsafe { std::mem::transmute(arr) }
        };

        Self {
            draw_pile,
            waste: FrenchDeck::new(),
            foundations: Foundations::default(),
            tableaus,
        }
    }
}

pub fn display<T>(game: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result
where
    T: Game,
{
    write!(f, "{: >3} ", game.draw_pile_size())?;
    match game.upturned() {
        None => write!(f, "___")?,
        Some(c) => write!(f, "{: >3}", c)?,
    }
    write!(f, "    ")?;
    use Suite::*;
    for suite in [Hearts, Diamonds, Clubs, Spades] {
        let foundation = game.foundations()[suite];
        match foundation {
            0 => write!(f, " __{}", suite)?,
            r => write!(f, " {: >3}", Card::new_unchecked(r, suite))?,
        }
    }
    writeln!(f)?;
    writeln!(f)?;
    for line in 0.. {
        #[derive(PartialEq)]
        enum PileState<'a> {
            Downturned,
            Upturned(&'a Card),
            Done,
        }

        let tableaus = game.tableaus();
        let pile_states = tableaus.iter().map(|t| {
            if line < t.downfaced_len {
                PileState::Downturned
            } else if line - t.downfaced_len < t.upturned.len() {
                PileState::Upturned(t.upturned.get(line - t.downfaced_len).unwrap())
            } else {
                PileState::Done
            }
        });

        if pile_states.clone().all(|s| s == PileState::Done) {
            break;
        }

        for s in pile_states {
            match s {
                PileState::Downturned => write!(f, "  ? ")?,
                PileState::Upturned(c) => write!(f, "{: >3} ", c)?,
                PileState::Done => write!(f, "    ")?,
            }
        }
        writeln!(f)?;
    }
    Ok(())
}

impl fmt::Display for MemoryGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display(self, f)
    }
}
