use crate::{
    cards::{Card, Deck, standard_deck},
    game::{Error, Event, Game, Outcome, Player, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const RESET_RANK: u8 = 2;
const BURN_RANK: u8 = 10;
const HAND_SIZE: usize = 3;
const MATCHING_CARDS_TO_BURN: usize = 4;

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
        let events = game.advance(0, &Action::Play { indices: vec![0] });
        assert!(events.contains(&Event::Burned));
        assert!(game.pile.is_empty());
        assert_eq!(game.turn.players, vec![0]);
        game.pile = vec![card(7), card(7)];
        game.players[0].hand = vec![card(7), card(7)];
        game.advance(
            0,
            &Action::Play {
                indices: vec![0, 1],
            },
        );
        assert!(game.pile.is_empty());
        assert_eq!(game.turn.players, vec![0]);
    }

    #[test]
    fn blind_failure_picks_up_without_exposing_other_down_cards() {
        let mut game = Shithead::new(2, [0; 32]).unwrap();
        game.deck = Deck::new(vec![]);
        game.pile = vec![card(14)];
        game.players[0] = PlayerCards {
            hand: vec![],
            face_up: vec![],
            face_down: vec![card(3), card(4)],
        };
        assert!(game.legal_actions(0).contains(&Action::Blind { index: 0 }));
        game.advance(0, &Action::Blind { index: 0 });
        assert_eq!(game.players[0].hand, vec![card(14), card(3)]);
        assert_eq!(game.view(1).players[0].down, 1);
        assert_eq!(game.turn.players, vec![1]);
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

    #[test]
    fn cards_are_played_from_hand_then_face_up_then_face_down() {
        let mut cards = PlayerCards {
            hand: vec![card(3)],
            face_up: vec![card(4)],
            face_down: vec![card(5)],
        };
        assert_eq!(cards.active_zone(), CardZone::Hand);
        assert_eq!(
            cards.take_play(&Action::Play { indices: vec![0] }),
            vec![card(3)]
        );
        assert_eq!(cards.active_zone(), CardZone::FaceUp);
        assert_eq!(
            cards.take_play(&Action::Play { indices: vec![0] }),
            vec![card(4)]
        );
        assert_eq!(cards.active_zone(), CardZone::FaceDown);
        assert_eq!(cards.take_play(&Action::Blind { index: 0 }), vec![card(5)]);
        assert!(cards.is_empty());
    }

    #[test]
    fn same_rank_subsets_preserve_action_and_removal_order() {
        let game = Shithead::new(2, [0; 32]).unwrap();
        let mut cards = PlayerCards {
            hand: vec![card(7), card(3), card(7), card(7)],
            face_up: vec![],
            face_down: vec![],
        };
        let indices: Vec<_> = game
            .same_rank_plays(&cards.hand)
            .into_iter()
            .map(|action| {
                let Action::Play { indices } = action else {
                    unreachable!()
                };
                indices
            })
            .collect();
        assert_eq!(
            indices,
            vec![
                vec![1],
                vec![0],
                vec![2],
                vec![0, 2],
                vec![3],
                vec![0, 3],
                vec![2, 3],
                vec![0, 2, 3]
            ]
        );
        cards.take_play(&Action::Play {
            indices: vec![0, 2, 3],
        });
        assert_eq!(cards.hand, vec![card(3)]);
    }

    #[test]
    fn finishing_on_a_burn_skips_finished_players_instead_of_taking_an_extra_turn() {
        let mut game = Shithead::new(3, [0; 32]).unwrap();
        game.deck = Deck::new(vec![]);
        game.players[0] = PlayerCards {
            hand: vec![card(BURN_RANK)],
            face_up: vec![],
            face_down: vec![],
        };
        game.players[1] = PlayerCards {
            hand: vec![],
            face_up: vec![],
            face_down: vec![],
        };
        game.finished = vec![1];
        let events = game.advance(0, &Action::Play { indices: vec![0] });
        assert!(events.contains(&Event::Burned));
        assert_eq!(game.finished, vec![1, 0]);
        assert_eq!(game.turn.players, vec![2]);
        assert_eq!(game.turn.number, 1);
        assert_eq!(game.outcome.unwrap().winners, vec![1, 0]);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerCards {
    hand: Vec<Card>,
    #[serde(rename = "up")]
    face_up: Vec<Card>,
    #[serde(rename = "down")]
    face_down: Vec<Card>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardZone {
    Hand,
    FaceUp,
    FaceDown,
}

impl PlayerCards {
    fn active_zone(&self) -> CardZone {
        if !self.hand.is_empty() {
            CardZone::Hand
        } else if !self.face_up.is_empty() {
            CardZone::FaceUp
        } else {
            CardZone::FaceDown
        }
    }

    fn cards(&self, zone: CardZone) -> &[Card] {
        match zone {
            CardZone::Hand => &self.hand,
            CardZone::FaceUp => &self.face_up,
            CardZone::FaceDown => &self.face_down,
        }
    }

    fn take_play(&mut self, action: &Action) -> Vec<Card> {
        match action {
            Action::Blind { index } => vec![self.face_down.remove(*index)],
            Action::Play { indices } => {
                let source = match self.active_zone() {
                    CardZone::Hand => &mut self.hand,
                    CardZone::FaceUp => &mut self.face_up,
                    CardZone::FaceDown => unreachable!("face-down cards use blind actions"),
                };
                // Remove from the end so earlier indices remain valid.
                indices
                    .iter()
                    .rev()
                    .map(|&index| source.remove(index))
                    .collect()
            }
            Action::Pickup => unreachable!("picking up does not play cards"),
        }
    }

    fn is_empty(&self) -> bool {
        self.hand.is_empty() && self.face_up.is_empty() && self.face_down.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shithead {
    players: Vec<PlayerCards>,
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
        card.rank == RESET_RANK
            || card.rank == BURN_RANK
            || self
                .pile
                .last()
                .is_none_or(|top| top.rank == RESET_RANK || card.rank >= top.rank)
    }

    fn same_rank_plays(&self, source: &[Card]) -> Vec<Action> {
        let mut actions = vec![];
        for rank in 2..=14 {
            let indices = source
                .iter()
                .enumerate()
                .filter(|(_, card)| card.rank == rank && self.playable(**card))
                .map(|(index, _)| index);
            // Each matching card doubles the subsets: without it, then with it.
            // This preserves the action order used by version 1 replays.
            let mut subsets = vec![vec![]];
            for index in indices {
                let with_card: Vec<_> = subsets
                    .iter()
                    .map(|subset| {
                        let mut subset = subset.clone();
                        subset.push(index);
                        subset
                    })
                    .collect();
                subsets.extend(with_card);
            }
            actions.extend(
                subsets
                    .into_iter()
                    .skip(1)
                    .map(|indices| Action::Play { indices }),
            );
        }
        actions
    }

    fn pile_has_four_of_a_kind(&self) -> bool {
        let Some(top) = self.pile.last() else {
            return false;
        };
        self.pile.len() >= MATCHING_CARDS_TO_BURN
            && self
                .pile
                .iter()
                .rev()
                .take(MATCHING_CARDS_TO_BURN)
                .all(|card| card.rank == top.rank)
    }

    fn resolve_play(&mut self, player: Player, played: Vec<Card>, events: &mut Vec<Event>) -> bool {
        let playable = self.playable(played[0]);
        let burns_pile = played[0].rank == BURN_RANK;
        self.pile.extend(played);
        if !playable {
            self.players[player].hand.append(&mut self.pile);
            return false;
        }
        if burns_pile || self.pile_has_four_of_a_kind() {
            self.burned += self.pile.len();
            self.pile.clear();
            events.push(Event::Burned);
            return true;
        }
        false
    }

    fn refill_hand(&mut self, player: Player) {
        while self.players[player].hand.len() < HAND_SIZE {
            let Some(card) = self.deck.draw() else {
                break;
            };
            self.players[player].hand.push(card);
        }
    }

    fn record_finish(&mut self, player: Player) {
        self.finished.push(player);
        if self.finished.len() == self.players.len() - 1 {
            self.outcome = Some(Outcome {
                winners: self.finished.clone(),
                scores: None,
            });
        }
    }

    fn next_turn(&mut self, extra_turn: bool) {
        if extra_turn {
            self.turn.number += 1;
        } else {
            self.turn.advance(self.players.len());
            while self.finished.contains(&self.turn.players[0]) {
                self.turn.players[0] = (self.turn.players[0] + 1) % self.players.len();
            }
        }
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
            .map(|_| PlayerCards {
                face_down: (0..HAND_SIZE).map(|_| deck.draw().unwrap()).collect(),
                face_up: (0..HAND_SIZE).map(|_| deck.draw().unwrap()).collect(),
                hand: (0..HAND_SIZE).map(|_| deck.draw().unwrap()).collect(),
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
        self.turn.clone()
    }
    fn outcome(&self) -> Option<Outcome> {
        self.outcome.clone()
    }
    fn legal_actions(&self, player: Player) -> Vec<Action> {
        if self.outcome.is_some() || !self.turn.players.contains(&player) {
            return vec![];
        }
        let cards = &self.players[player];
        let zone = cards.active_zone();
        let mut actions = if zone == CardZone::FaceDown {
            (0..cards.face_down.len())
                .map(|index| Action::Blind { index })
                .collect()
        } else {
            self.same_rank_plays(cards.cards(zone))
        };
        if !self.pile.is_empty() {
            actions.push(Action::Pickup);
        }
        actions
    }
    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        let mut events = vec![Event::Played { player }];
        let mut extra_turn = false;
        match action {
            Action::Pickup => self.players[player].hand.append(&mut self.pile),
            Action::Blind { .. } | Action::Play { .. } => {
                let played = self.players[player].take_play(action);
                extra_turn = self.resolve_play(player, played, &mut events);
                self.refill_hand(player);
                if self.players[player].is_empty() {
                    self.record_finish(player);
                    extra_turn = false;
                }
            }
        }
        self.next_turn(extra_turn);
        events
    }
    fn view(&self, viewer: Player) -> View {
        View {
            hand: self
                .players
                .get(viewer)
                .map(|cards| cards.hand.clone())
                .unwrap_or_default(),
            players: self
                .players
                .iter()
                .map(|cards| PlayerView {
                    count: cards.hand.len(),
                    up: cards.face_up.clone(),
                    down: cards.face_down.len(),
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
