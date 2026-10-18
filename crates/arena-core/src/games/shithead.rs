use crate::{
    cards::{Card, Deck, standard_deck},
    game::{Error, Event, Game, Outcome, Player, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schema(as = ShitheadAction)]
pub enum Action {
    Play { indices: Vec<usize> },
    Blind { index: usize },
    Pickup,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Session, cards::Suit};

    fn card(rank: u8) -> Card {
        Card {
            rank,
            suit: Suit::Clubs,
        }
    }

    #[test]
    fn reset_burn_and_four_of_a_kind() {
        let mut game = Shithead::new(2, [0; 32]).unwrap();
        game.pile = vec![card(14)];
        assert!(game.playable(card(2)));
        assert!(game.playable(card(10)));
        assert!(!game.playable(card(13)));
        game.pile.push(card(2));
        assert!(game.playable(card(3)));
        game.players[0].hand = vec![card(10)];
        let events = game.advance(&Action::Play { indices: vec![0] });
        assert!(events.contains(&Event::Burned));
        assert!(game.pile.is_empty());
        assert_eq!(game.turn.player, 0);
        game.pile = vec![card(7), card(7)];
        game.players[0].hand = vec![card(7), card(7)];
        game.advance(&Action::Play {
            indices: vec![0, 1],
        });
        assert!(game.pile.is_empty());
        assert_eq!(game.turn.player, 0);
    }

    #[test]
    fn blind_failure_picks_up_without_exposing_other_down_cards() {
        let mut game = Shithead::new(2, [0; 32]).unwrap();
        game.deck = Deck::new(vec![]);
        game.pile = vec![card(14)];
        game.players[0] = Cards {
            hand: vec![],
            up: vec![],
            down: vec![card(3), card(4)],
        };
        assert!(game.legal_actions().contains(&Action::Blind { index: 0 }));
        game.advance(&Action::Blind { index: 0 });
        assert_eq!(game.players[0].hand, vec![card(14), card(3)]);
        assert_eq!(game.view(1).players[0].down, 1);
        assert_eq!(game.turn.player, 1);
    }

    #[test]
    fn repeated_or_unsorted_indices_are_rejected() {
        let mut game = Session::<Shithead>::new(2, [0; 32]).unwrap();
        let before = serde_json::to_value(&game).unwrap();
        assert_eq!(
            game.act(
                0,
                Action::Play {
                    indices: vec![0, 0]
                }
            )
            .unwrap_err(),
            Error::IllegalMove
        );
        assert_eq!(
            game.act(
                0,
                Action::Play {
                    indices: vec![1, 0]
                }
            )
            .unwrap_err(),
            Error::IllegalMove
        );
        assert_eq!(serde_json::to_value(game).unwrap(), before);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cards {
    hand: Vec<Card>,
    up: Vec<Card>,
    down: Vec<Card>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shithead {
    players: Vec<Cards>,
    deck: Deck<Card>,
    pile: Vec<Card>,
    burned: usize,
    finished: Vec<Player>,
    turn: Turn,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = ShitheadPlayerView)]
pub struct PlayerView {
    pub count: usize,
    pub up: Vec<Card>,
    pub down: usize,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = ShitheadView)]
pub struct View {
    pub hand: Vec<Card>,
    pub players: Vec<PlayerView>,
    pub top: Option<Card>,
    pub pile: usize,
    pub stock: usize,
    pub burned: usize,
    pub finished: Vec<Player>,
}

impl Shithead {
    fn playable(&self, card: Card) -> bool {
        card.rank == 2
            || card.rank == 10
            || self
                .pile
                .last()
                .is_none_or(|top| top.rank == 2 || card.rank >= top.rank)
    }
}

impl Game for Shithead {
    type Action = Action;
    type Event = Event;
    type View = View;
    fn new(players: usize, seed: Seed) -> Result<Self, Error> {
        if !(2..=5).contains(&players) {
            return Err(Error::PlayerCount);
        }
        let mut deck = standard_deck();
        deck.shuffle(&mut ChaCha12Rng::from_seed(seed));
        let players = (0..players)
            .map(|_| Cards {
                down: (0..3).map(|_| deck.draw().unwrap()).collect(),
                up: (0..3).map(|_| deck.draw().unwrap()).collect(),
                hand: (0..3).map(|_| deck.draw().unwrap()).collect(),
            })
            .collect();
        Ok(Self {
            players,
            deck,
            pile: vec![],
            burned: 0,
            finished: vec![],
            turn: Turn::first(),
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
        let cards = &self.players[self.turn.player];
        let mut actions = if cards.hand.is_empty() && cards.up.is_empty() {
            (0..cards.down.len())
                .map(|index| Action::Blind { index })
                .collect()
        } else {
            let source = if cards.hand.is_empty() {
                &cards.up
            } else {
                &cards.hand
            };
            let mut actions = vec![];
            for rank in 2..=14 {
                let indices: Vec<_> = source
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.rank == rank && self.playable(**c))
                    .map(|(i, _)| i)
                    .collect();
                for mask in 1..(1usize << indices.len()) {
                    actions.push(Action::Play {
                        indices: indices
                            .iter()
                            .enumerate()
                            .filter(|(bit, _)| mask & (1 << bit) != 0)
                            .map(|(_, &i)| i)
                            .collect(),
                    });
                }
            }
            actions
        };
        if !self.pile.is_empty() {
            actions.push(Action::Pickup);
        }
        actions
    }
    fn advance(&mut self, action: &Action) -> Vec<Event> {
        let player = self.turn.player;
        let mut events = vec![Event::Played { player }];
        let mut again = false;
        match action {
            Action::Pickup => self.players[player].hand.append(&mut self.pile),
            _ => {
                let played = match action {
                    Action::Blind { index } => vec![self.players[player].down.remove(*index)],
                    Action::Play { indices } => {
                        let cards = &mut self.players[player];
                        let source = if cards.hand.is_empty() {
                            &mut cards.up
                        } else {
                            &mut cards.hand
                        };
                        indices
                            .iter()
                            .rev()
                            .map(|&i| source.remove(i))
                            .collect::<Vec<_>>()
                    }
                    Action::Pickup => unreachable!(),
                };
                let valid = self.playable(played[0]);
                let ten = played[0].rank == 10;
                self.pile.extend(played);
                if !valid {
                    self.players[player].hand.append(&mut self.pile);
                } else if ten
                    || (self.pile.len() >= 4
                        && self
                            .pile
                            .iter()
                            .rev()
                            .take(4)
                            .all(|c| c.rank == self.pile.last().unwrap().rank))
                {
                    self.burned += self.pile.len();
                    self.pile.clear();
                    again = true;
                    events.push(Event::Burned);
                }
                while self.players[player].hand.len() < 3 {
                    match self.deck.draw() {
                        Some(card) => self.players[player].hand.push(card),
                        None => break,
                    }
                }
                let cards = &self.players[player];
                if cards.hand.is_empty() && cards.up.is_empty() && cards.down.is_empty() {
                    self.finished.push(player);
                    again = false;
                    if self.finished.len() == self.players.len() - 1 {
                        self.outcome = Some(Outcome {
                            winners: self.finished.clone(),
                            scores: None,
                        });
                    }
                }
            }
        }
        if again {
            self.turn.number += 1;
        } else {
            self.turn.next(self.players.len());
            while self.finished.contains(&self.turn.player) {
                self.turn.player = (self.turn.player + 1) % self.players.len();
            }
        }
        events
    }
    fn view(&self, viewer: Player) -> View {
        View {
            hand: self
                .players
                .get(viewer)
                .map(|c| c.hand.clone())
                .unwrap_or_default(),
            players: self
                .players
                .iter()
                .map(|c| PlayerView {
                    count: c.hand.len(),
                    up: c.up.clone(),
                    down: c.down.len(),
                })
                .collect(),
            top: self.pile.last().copied(),
            pile: self.pile.len(),
            stock: self.deck.len(),
            burned: self.burned,
            finished: self.finished.clone(),
        }
    }
}
