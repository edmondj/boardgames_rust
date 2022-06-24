use crate::cards;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Suite {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Color {
    Red,
    Black,
}

impl Suite {
    pub fn color(&self) -> Color {
        use Color::*;
        use Suite::*;
        match self {
            Hearts | Diamonds => Red,
            Clubs | Spades => Black,
        }
    }
}

impl fmt::Display for Suite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Suite::*;
        write!(
            f,
            "{}",
            match self {
                Hearts => "♥",
                Diamonds => "♦",
                Clubs => "♣",
                Spades => "♠",
            }
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Card {
    rank: u8,
    suite: Suite,
}

struct Rank(u8);

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 || self.0 > KING {
            panic!("Invalid rank value")
        }
        match self.0 {
            ACE => write!(f, "A"),
            JACK => write!(f, "J"),
            QUEEN => write!(f, "Q"),
            KING => write!(f, "K"),
            rank => write!(f, "{}", rank),
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&format!("{}{}", Rank(self.rank), self.suite))
    }
}

pub const ACE: u8 = 1;
pub const JACK: u8 = 11;
pub const QUEEN: u8 = 12;
pub const KING: u8 = 13;

impl Card {
    pub fn new_unchecked(rank: u8, suite: Suite) -> Self {
        Self { rank, suite }
    }

    pub fn new(rank: u8, suite: Suite) -> Self {
        if rank < ACE || rank > KING {
            panic!("Invalid card rank")
        }
        Self::new_unchecked(rank, suite)
    }

    pub fn rank(&self) -> u8 {
        self.rank
    }

    pub fn suite(&self) -> Suite {
        self.suite
    }
}

struct StandardDeck {
    cur: Option<Card>,
}

impl StandardDeck {
    fn new() -> Self {
        StandardDeck { cur: None }
    }
}

impl Iterator for StandardDeck {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        use Suite::*;
        self.cur = match self.cur {
            None => Some(Card::new_unchecked(1, Hearts)),
            Some(c) => {
                if c.rank() == KING {
                    match c.suite() {
                        Hearts => Some(Card::new_unchecked(ACE, Diamonds)),
                        Diamonds => Some(Card::new_unchecked(ACE, Clubs)),
                        Clubs => Some(Card::new_unchecked(ACE, Spades)),
                        Spades => None,
                    }
                } else {
                    Some(Card::new_unchecked(c.rank() + 1, c.suite()))
                }
            }
        };
        self.cur
    }
}

pub fn standard_52_deck() -> cards::Deck<Card> {
    StandardDeck::new().collect()
}

pub fn standard_32_deck() -> cards::Deck<Card> {
    StandardDeck::new()
        .filter(|c| c.rank() < 2 || c.rank() > 6)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn fmt() {
        assert_eq!(Card::new_unchecked(ACE, Suite::Spades).to_string(), "A♠");
        assert_eq!(Card::new_unchecked(8, Suite::Hearts).to_string(), "8♥");
        assert_eq!(Card::new_unchecked(JACK, Suite::Clubs).to_string(), "J♣");
        assert_eq!(Card::new_unchecked(QUEEN, Suite::Hearts).to_string(), "Q♥");
        assert_eq!(Card::new_unchecked(KING, Suite::Diamonds).to_string(), "K♦");
    }

    #[test]
    pub fn std() {
        assert_eq!(standard_52_deck().len(), 52);
        assert_eq!(standard_32_deck().len(), 32);
    }
}
