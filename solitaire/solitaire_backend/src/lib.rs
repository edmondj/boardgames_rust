use boards::cards::french::{standard_52_deck, Card, Suite, KING};
use boards::cards::FrenchDeck;
use boards::random_engine::RandomEngine;
use core::fmt;
use std::mem::MaybeUninit;

#[derive(Debug, Clone, Default)]
pub struct Tableau {
    pile: Vec<Card>,
    upturned: usize,
}

impl Tableau {
    pub fn len(&self) -> usize {
        self.pile.len()
    }

    pub fn upturned_len(&self) -> usize {
        self.upturned
    }

    pub fn downfaced_len(&self) -> usize {
        self.len() - self.upturned_len()
    }

    pub fn upturned(&self, i: usize) -> Option<&Card> {
        if i < self.upturned {
            Some(&self.pile[self.downfaced_len() + i])
        } else {
            None
        }
    }

    pub fn bottom(&self) -> Option<&Card> {
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

pub type Foundations = [u8; 4];
pub const TABLEAUS_COUNT: usize = 7;
pub type Tableaus = [Tableau; TABLEAUS_COUNT];

pub struct State {
    draw_pile: FrenchDeck,
    waste: FrenchDeck,
    foundations: Foundations,
    tableaus: Tableaus,
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

pub enum ActionResult {
    Victory,
    OnGoing,
    Failed(String),
}

impl State {
    pub fn new(rand: &mut impl RandomEngine) -> Self {
        let mut draw_pile = standard_52_deck();
        FrenchDeck::shuffle(&mut draw_pile, rand);

        let tableaus = {
            let mut arr: [MaybeUninit<Tableau>; TABLEAUS_COUNT] =
                unsafe { MaybeUninit::uninit().assume_init() };
            for i in 0..arr.len() {
                arr[i].write(Tableau {
                    pile: draw_pile.draw_many(i + 1).collect(),
                    upturned: 1,
                });
            }
            unsafe { std::mem::transmute(arr) }
        };

        Self {
            draw_pile,
            waste: FrenchDeck::new(),
            foundations: [0; 4],
            tableaus,
        }
    }

    pub fn draw_pile(&self) -> &FrenchDeck {
        &self.draw_pile
    }

    pub fn upturned(&self) -> Option<&Card> {
        self.waste.peek()
    }

    pub fn waste_len(&self) -> usize {
        self.waste.len().saturating_sub(1)
    }

    pub fn foundations(&self) -> &Foundations {
        &self.foundations
    }

    pub fn tableaus(&self) -> &Tableaus {
        &self.tableaus
    }

    pub fn act(&mut self, action: Action) -> ActionResult {
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
                    use Suite::*;
                    let dest: &mut u8 = match c.suite() {
                        Hearts => &mut self.foundations[0],
                        Diamonds => &mut self.foundations[1],
                        Clubs => &mut self.foundations[2],
                        Spades => &mut self.foundations[3],
                    };
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
                        if self.foundations.iter().all(|r| r == &KING) {
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
                    Upturned => self.upturned(),
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

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{: >3} ", self.draw_pile().len())?;
        match self.upturned() {
            None => write!(f, "___")?,
            Some(c) => write!(f, "{: >3}", c)?,
        }
        write!(f, "    ")?;
        for i in 0..self.foundations().len() {
            let foundation = self.foundations()[i];
            use Suite::*;
            let suite = match i {
                0 => Hearts,
                1 => Diamonds,
                2 => Clubs,
                3 => Spades,
                _ => panic!(),
            };
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

            let pile_states = self.tableaus().iter().map(|t| {
                if line < t.downfaced_len() {
                    PileState::Downturned
                } else if line < t.len() {
                    PileState::Upturned(t.upturned(line - t.downfaced_len()).unwrap())
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
}
