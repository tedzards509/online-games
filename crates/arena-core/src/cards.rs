//! Standard playing cards and a shuffled draw pile for any card type.

use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One of the four standard playing-card suits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    /// Suits in the fixed order used to construct standard decks.
    pub const ALL: [Self; 4] = [Self::Clubs, Self::Diamonds, Self::Hearts, Self::Spades];
}

/// A standard playing card; ranks run from two (2) to ace (14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Card {
    /// The card's printed suit.
    pub suit: Suit,
    /// The card's rank, from 2 to 14.
    pub rank: u8,
}

/// Builds an unshuffled 52-card deck in suit-then-rank order, without jokers.
pub fn standard_deck() -> Deck<Card> {
    Deck::new(
        Suit::ALL
            .into_iter()
            .flat_map(|suit| (2..=14).map(move |rank| Card { suit, rank }))
            .collect(),
    )
}

/// A draw pile whose top is the last element of its backing vector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck<C>(Vec<C>);

impl<C> Deck<C> {
    /// Creates a pile without changing the supplied card order.
    pub fn new(cards: Vec<C>) -> Self {
        Self(cards)
    }
    /// Counts the cards remaining in the pile.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Reports whether no cards remain.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Removes the top card, or returns `None` for an empty pile.
    pub fn draw(&mut self) -> Option<C> {
        self.0.pop()
    }
    /// Borrows the top card without removing it.
    pub fn peek(&self) -> Option<&C> {
        self.0.last()
    }
    /// Places a card on top of the pile.
    pub fn push(&mut self, card: C) {
        self.0.push(card);
    }
    /// Adds cards in iteration order; the last added card becomes the top.
    pub fn extend(&mut self, cards: impl IntoIterator<Item = C>) {
        self.0.extend(cards);
    }
    /// Shuffles in place using the caller's random generator.
    pub fn shuffle(&mut self, rng: &mut impl Rng) {
        self.0.shuffle(rng);
    }
    /// When empty, shuffles discards into the pile while keeping their top visible.
    pub fn recycle(&mut self, discard: &mut Vec<C>, rng: &mut impl Rng) {
        if self.is_empty() && discard.len() > 1 {
            let top = discard.pop().expect("nonempty discard");
            self.0.append(discard);
            discard.push(top);
            self.shuffle(rng);
        }
    }
}
