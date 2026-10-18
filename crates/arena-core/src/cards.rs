use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Self; 4] = [Self::Clubs, Self::Diamonds, Self::Hearts, Self::Spades];
}

/// Ranks run from two (2) to ace (14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Card {
    pub suit: Suit,
    pub rank: u8,
}

pub fn standard_deck() -> Deck<Card> {
    Deck::new(
        Suit::ALL
            .into_iter()
            .flat_map(|suit| (2..=14).map(move |rank| Card { suit, rank }))
            .collect(),
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck<C>(Vec<C>);

impl<C> Deck<C> {
    pub fn new(cards: Vec<C>) -> Self {
        Self(cards)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn draw(&mut self) -> Option<C> {
        self.0.pop()
    }
    pub fn peek(&self) -> Option<&C> {
        self.0.last()
    }
    pub fn push(&mut self, card: C) {
        self.0.push(card);
    }
    pub fn extend(&mut self, cards: impl IntoIterator<Item = C>) {
        self.0.extend(cards);
    }
    pub fn shuffle(&mut self, rng: &mut impl Rng) {
        self.0.shuffle(rng);
    }
    /// Keeps the visible discard and recycles everything beneath it.
    pub fn recycle(&mut self, discard: &mut Vec<C>, rng: &mut impl Rng) {
        if self.is_empty() && discard.len() > 1 {
            let top = discard.pop().expect("nonempty discard");
            self.0.append(discard);
            discard.push(top);
            self.shuffle(rng);
        }
    }
}
