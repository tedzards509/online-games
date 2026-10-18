use crate::{
    cards::{Card, Deck, Suit, standard_deck},
    game::{Error, Event, Game, Outcome, Player, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schema(as = CrazyEightsAction)]
pub enum Action {
    Play { index: usize, suit: Option<Suit> },
    Draw,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(rank: u8, suit: Suit) -> Card {
        Card { rank, suit }
    }

    #[test]
    fn eight_changes_suit_and_other_special_ranks_do_not() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        game.hands[0] = vec![card(8, Suit::Clubs), card(3, Suit::Clubs)];
        game.discard = vec![card(14, Suit::Hearts)];
        game.suit = Suit::Hearts;
        assert_eq!(game.legal_actions().len(), 4);
        game.advance(&Action::Play {
            index: 0,
            suit: Some(Suit::Diamonds),
        });
        assert_eq!(game.suit, Suit::Diamonds);
        game.hands[1] = vec![card(2, Suit::Diamonds), card(11, Suit::Spades)];
        assert_eq!(
            game.legal_actions(),
            vec![Action::Play {
                index: 0,
                suit: None
            }]
        );
        game.advance(&Action::Play {
            index: 0,
            suit: None,
        });
        assert_eq!(game.turn.player, 0);
    }

    #[test]
    fn exhausted_stock_passes_until_everyone_is_blocked() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        game.deck = Deck::new(vec![]);
        game.discard = vec![card(3, Suit::Hearts)];
        game.suit = Suit::Hearts;
        game.hands = vec![vec![card(4, Suit::Clubs)], vec![card(5, Suit::Spades)]];
        assert_eq!(game.legal_actions(), vec![Action::Draw]);
        game.advance(&Action::Draw);
        assert!(game.outcome.is_none());
        game.advance(&Action::Draw);
        assert!(game.outcome.unwrap().winners.is_empty());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrazyEights {
    hands: Vec<Vec<Card>>,
    deck: Deck<Card>,
    discard: Vec<Card>,
    suit: Suit,
    rng: ChaCha12Rng,
    turn: Turn,
    blocked: usize,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = CrazyEightsView)]
pub struct View {
    pub hand: Vec<Card>,
    pub counts: Vec<usize>,
    pub top: Card,
    pub suit: Suit,
    pub stock: usize,
}

impl Game for CrazyEights {
    type Action = Action;
    type Event = Event;
    type View = View;
    fn new(players: usize, seed: Seed) -> Result<Self, Error> {
        if !(2..=6).contains(&players) {
            return Err(Error::PlayerCount);
        }
        let mut rng = ChaCha12Rng::from_seed(seed);
        let mut deck = standard_deck();
        deck.shuffle(&mut rng);
        let hands = (0..players)
            .map(|_| {
                (0..if players == 2 { 7 } else { 5 })
                    .map(|_| deck.draw().unwrap())
                    .collect()
            })
            .collect();
        let top = deck.draw().unwrap();
        Ok(Self {
            hands,
            deck,
            discard: vec![top],
            suit: top.suit,
            rng,
            turn: Turn::first(),
            blocked: 0,
            outcome: None,
        })
    }
    fn turn(&self) -> Turn {
        self.turn
    }
    fn outcome(&self) -> Option<Outcome> {
        self.outcome.clone()
    }
    fn legal_actions(&self) -> Vec<Action> {
        if self.outcome.is_some() {
            return vec![];
        }
        let top = self.discard.last().unwrap();
        let actions: Vec<_> = self.hands[self.turn.player]
            .iter()
            .enumerate()
            .flat_map(|(index, card)| {
                if card.rank == 8 {
                    Suit::ALL
                        .into_iter()
                        .map(|suit| Action::Play {
                            index,
                            suit: Some(suit),
                        })
                        .collect()
                } else if card.suit == self.suit || card.rank == top.rank {
                    vec![Action::Play { index, suit: None }]
                } else {
                    vec![]
                }
            })
            .collect();
        if actions.is_empty() {
            vec![Action::Draw]
        } else {
            actions
        }
    }
    fn advance(&mut self, action: &Action) -> Vec<Event> {
        let player = self.turn.player;
        match action {
            Action::Play { index, suit } => {
                let card = self.hands[player].remove(*index);
                self.suit = suit.unwrap_or(card.suit);
                self.discard.push(card);
                self.blocked = 0;
                if self.hands[player].is_empty() {
                    self.outcome = Some(Outcome {
                        winners: vec![player],
                        scores: None,
                    });
                }
            }
            Action::Draw => {
                self.deck.recycle(&mut self.discard, &mut self.rng);
                if let Some(card) = self.deck.draw() {
                    self.hands[player].push(card);
                    self.blocked = 0;
                } else {
                    self.blocked += 1;
                }
                if self.blocked == self.hands.len() {
                    self.outcome = Some(Outcome {
                        winners: vec![],
                        scores: None,
                    });
                }
            }
        }
        self.turn.next(self.hands.len());
        vec![Event::Played { player }]
    }
    fn view(&self, viewer: Player) -> View {
        View {
            hand: self.hands.get(viewer).cloned().unwrap_or_default(),
            counts: self.hands.iter().map(Vec::len).collect(),
            top: *self.discard.last().unwrap(),
            suit: self.suit,
            stock: self.deck.len(),
        }
    }
}
