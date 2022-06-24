use crate::random_engine::{to_rng_core, RandomEngine};
use rand::distributions::Distribution;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Deck<T> {
    cards: VecDeque<T>,
}

impl<T> Deck<T> {
    pub fn new() -> Self {
        Self {
            cards: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn draw(&mut self) -> Option<T> {
        if self.cards.is_empty() {
            None
        } else {
            self.cards.pop_front()
        }
    }

    pub fn draw_many(&mut self, size: usize) -> impl DoubleEndedIterator<Item = T> + '_ {
        self.cards.drain(0..size)
    }

    pub fn draw_all(&mut self) -> impl DoubleEndedIterator<Item = T> + '_ {
        self.draw_many(self.len())
    }

    pub fn peek(&self) -> Option<&T> {
        self.cards.front()
    }

    pub fn peek_many(&self, count: usize) -> impl Iterator<Item = &T> {
        self.cards.iter().take(count)
    }

    pub fn put_top(&mut self, card: T) {
        self.cards.push_front(card)
    }

    pub fn put_top_many(
        &mut self,
        cards: impl IntoIterator<IntoIter = impl DoubleEndedIterator<Item = T>>,
    ) {
        for card in cards.into_iter().rev() {
            self.put_top(card);
        }
    }

    pub fn put_bottom(&mut self, card: T) {
        self.cards.push_back(card)
    }

    pub fn put_bottom_many(
        &mut self,
        cards: impl IntoIterator<IntoIter = impl Iterator<Item = T>>,
    ) {
        for card in cards.into_iter() {
            self.put_bottom(card);
        }
    }

    pub fn shuffle(deck: &mut Deck<T>, r: &mut impl RandomEngine) {
        let len = deck.cards.len();
        let mut rng = to_rng_core(r);
        for i in (1..len).rev() {
            let j = rand::distributions::Uniform::new_inclusive(0, i).sample(&mut rng);
            deck.cards.swap(i, j);
        }
    }
}

impl<T> FromIterator<T> for Deck<T> {
    fn from_iter<Iter>(iter: Iter) -> Self
    where
        Iter: std::iter::IntoIterator<Item = T>,
    {
        Self {
            cards: iter.into_iter().collect(),
        }
    }
}

impl<T> Default for Deck<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub mod french;
pub type FrenchDeck = Deck<french::Card>;

#[cfg(test)]
use crate::random_engine::XorShifEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw() {
        let mut deck = (1..6).collect::<Deck<_>>();

        assert_eq!(deck.draw(), Some(1));
        assert_eq!(deck.draw_many(3).collect::<Vec<_>>(), vec![2, 3, 4]);
        assert_eq!(deck.len(), 1);
    }

    #[test]
    fn shuffle() {
        let mut deck = (1..10).collect::<Deck<_>>();
        let mut rng = XorShifEngine::new(0);

        Deck::shuffle(&mut deck, &mut rng);
        assert_eq!(deck.cards, vec![2, 3, 4, 5, 6, 7, 8, 9, 1]);
    }

    #[test]
    fn put_top() {
        let mut deck = (1..5).collect::<Deck<_>>();

        deck.put_top(5);
        assert_eq!(deck.cards, [5, 1, 2, 3, 4]);
        assert_eq!(deck.peek(), Some(&5));

        deck.put_top_many(1..3);
        assert_eq!(deck.cards, [1, 2, 5, 1, 2, 3, 4]);
        assert_eq!(deck.peek_many(3).collect::<Vec<_>>(), vec![&1, &2, &5]);
    }

    #[test]
    fn put_bottom() {
        let mut deck = (1..5).collect::<Deck<_>>();

        deck.put_bottom(5);
        assert_eq!(deck.cards, [1, 2, 3, 4, 5]);

        deck.put_bottom_many(1..3);
        assert_eq!(deck.cards, [1, 2, 3, 4, 5, 1, 2]);
    }
}
