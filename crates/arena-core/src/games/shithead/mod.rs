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
const INITIAL_HAND_SIZE: usize = 7;

/// Whether four matching cards may bypass the ordinary rank restrictions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = ShitheadGameConfig)]
pub struct GameConfig {
    pub four_card_bombs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schema(as = ShitheadAction)]
pub enum Action {
    SelectFaceUp { indices: Vec<usize> },
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

    fn playing_session(players: usize) -> Session<Shithead> {
        let mut session = Session::<Shithead>::new(players, [0; 32]).unwrap();
        for player in 0..players {
            session
                .act(
                    player,
                    Action::SelectFaceUp {
                        indices: vec![0, 1, 2],
                    },
                )
                .unwrap();
        }
        session
    }

    fn playing_game(players: usize) -> Shithead {
        playing_session(players).state
    }

    #[test]
    fn face_up_selection_is_simultaneous_and_starts_play_only_when_everyone_is_ready() {
        let mut session = Session::<Shithead>::new(3, [0; 32]).unwrap();
        for (player, eligible) in [(2, vec![0, 1]), (0, vec![1]), (1, vec![0])] {
            assert_eq!(session.state.legal_actions(player).len(), 35);
            let hand = session.state.players[player].hand.clone();
            session
                .act(
                    player,
                    Action::SelectFaceUp {
                        indices: vec![1, 3, 6],
                    },
                )
                .unwrap();
            assert_eq!(
                session.state.players[player].face_up,
                vec![hand[1], hand[3], hand[6]]
            );
            assert_eq!(session.state.players[player].hand.len(), 4);
            assert_eq!(session.state.turn.players, eligible);
            if player != 1 {
                assert!(session.state.setup);
                let before = serde_json::to_value(&session).unwrap();
                assert_eq!(
                    session
                        .act(
                            player,
                            Action::SelectFaceUp {
                                indices: vec![0, 1, 2]
                            }
                        )
                        .unwrap_err(),
                    Error::NotYourTurn
                );
                assert_eq!(serde_json::to_value(&session).unwrap(), before);
            }
        }
        assert!(!session.state.setup);
        assert_eq!(session.state.turn.players, vec![0]);
        assert!(
            session
                .state
                .legal_actions(0)
                .iter()
                .all(|a| matches!(a, Action::Play { .. }))
        );
    }

    #[test]
    fn setup_rejects_nontriples_invalid_indices_and_normal_play_without_mutation() {
        let mut session = Session::<Shithead>::new(2, [0; 32]).unwrap();
        let before = serde_json::to_value(&session).unwrap();
        for action in [
            Action::SelectFaceUp {
                indices: vec![0, 1],
            },
            Action::SelectFaceUp {
                indices: vec![0, 1, 2, 3],
            },
            Action::SelectFaceUp {
                indices: vec![0, 0, 1],
            },
            Action::SelectFaceUp {
                indices: vec![2, 1, 0],
            },
            Action::SelectFaceUp {
                indices: vec![0, 1, usize::MAX],
            },
            Action::Play { indices: vec![0] },
            Action::Blind { index: 0 },
            Action::Pickup,
        ] {
            assert_eq!(session.act(1, action).unwrap_err(), Error::IllegalMove);
            assert_eq!(serde_json::to_value(&session).unwrap(), before);
        }
    }

    #[test]
    fn threes_are_transparent_and_sevens_restrict_even_tens() {
        let mut game = playing_game(2);
        for beneath in [7, 14] {
            game.pile = vec![card(beneath), card(3), card(3)];
            assert_eq!(game.view(0).top, Some(card(3)));
            assert_eq!(game.view(0).effective_top, Some(card(beneath)));
            assert!(game.playable(card(3)));
            assert_eq!(game.playable(card(10)), beneath != 7);
            assert_eq!(game.playable(card(7)), beneath == 7);
        }
        game.pile = vec![card(3), card(3)];
        assert_eq!(game.effective_top(), None);
        assert!((2..=14).all(|rank| game.playable(card(rank))));
    }

    #[test]
    fn bombs_bypass_rank_restrictions_only_when_enabled() {
        for enabled in [false, true] {
            for (top, rank) in [(7, 10), (14, 4)] {
                let mut session = playing_session(2);
                session.state.config.four_card_bombs = enabled;
                session.state.pile = vec![card(top)];
                session.state.players[0].hand = vec![card(rank); 4];
                let action = Action::Play {
                    indices: vec![0, 1, 2, 3],
                };
                assert_eq!(session.state.legal_actions(0).contains(&action), enabled);
                assert!(!session.state.legal_actions(0).contains(&Action::Play {
                    indices: vec![0, 1, 2]
                }));
                let before = serde_json::to_value(&session).unwrap();
                if enabled {
                    assert!(
                        session
                            .act(0, action)
                            .unwrap()
                            .events
                            .contains(&Event::Burned)
                    );
                    assert!(session.state.pile.is_empty());
                    assert_eq!(session.state.burned, 5);
                    assert_eq!(session.state.turn.players, vec![0]);
                } else {
                    assert_eq!(session.act(0, action).unwrap_err(), Error::IllegalMove);
                    assert_eq!(serde_json::to_value(&session).unwrap(), before);
                }
            }
        }
    }

    #[test]
    fn setup_deals_seven_hand_cards_and_three_down_with_private_values_hidden() {
        for players in 2..=5 {
            let game = Shithead::new(players, [0; 32]).unwrap();
            assert_eq!(game.turn().players, (0..players).collect::<Vec<_>>());
            assert!(game.setup);
            assert!(game.pile.is_empty());
            assert_eq!(game.burned, 0);
            assert!(game.finished.is_empty());
            assert!(game.outcome().is_none());
            assert_eq!(game.deck.len(), 52 - 10 * players);
            let mut cards = vec![];
            let mut stock = game.deck.clone();
            while let Some(card) = stock.draw() {
                cards.push(card);
            }
            for (player, zones) in game.players.iter().enumerate() {
                assert_eq!(
                    (zones.hand.len(), zones.face_up.len(), zones.face_down.len()),
                    (7, 0, 3)
                );
                cards.extend(
                    zones
                        .hand
                        .iter()
                        .chain(&zones.face_up)
                        .chain(&zones.face_down)
                        .copied(),
                );
                let view = game.view(player);
                assert_eq!(view.hand, zones.hand);
                for (other, public) in view.players.iter().enumerate() {
                    assert_eq!(public.count, 7);
                    assert_eq!(public.down, 3);
                    assert_eq!(public.up, game.players[other].face_up);
                }
                let json = serde_json::to_value(view).unwrap();
                assert!(json.get("deck").is_none());
                assert!(
                    json["players"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|p| p.get("hand").is_none() && p["down"] == 3)
                );
            }
            assert_eq!(cards.len(), 52);
            for rank in 2..=14 {
                for suit in Suit::ALL {
                    assert_eq!(
                        cards.iter().filter(|&&c| c == Card { rank, suit }).count(),
                        1
                    );
                }
            }
        }
    }

    #[test]
    fn rank_legality_matches_the_rules_for_every_rank_and_top() {
        let mut game = playing_game(2);
        for top in 2..=14 {
            game.pile = vec![card(top)];
            for rank in 2..=14 {
                for suit in Suit::ALL {
                    assert_eq!(
                        game.playable(Card { rank, suit }),
                        rank == 2
                            || rank == 3
                            || top == 2
                            || top == 3
                            || if top == 7 {
                                rank <= 7
                            } else {
                                rank == 10 || rank >= top
                            }
                    );
                }
            }
        }
        game.pile.clear();
        assert!((2..=14).all(|rank| game.playable(card(rank))));
    }

    #[test]
    fn pickup_is_optional_with_a_play_mandatory_without_one_and_illegal_on_empty_pile() {
        let mut session = playing_session(2);
        session.state.pile = vec![card(6)];
        session.state.players[0].hand = vec![card(8)];
        assert_eq!(
            session.state.legal_actions(0),
            vec![Action::Play { indices: vec![0] }, Action::Pickup]
        );
        let stock = session.state.deck.len();
        session.act(0, Action::Pickup).unwrap();
        assert_eq!(session.state.players[0].hand, vec![card(8), card(6)]);
        assert!(session.state.pile.is_empty());
        assert_eq!(session.state.deck.len(), stock);
        assert_eq!(session.state.turn.players, vec![1]);
        let before = serde_json::to_value(&session).unwrap();
        assert_eq!(
            session.act(1, Action::Pickup).unwrap_err(),
            Error::IllegalMove
        );
        assert_eq!(serde_json::to_value(&session).unwrap(), before);
        session.state.pile = vec![card(14)];
        session.state.players[1].hand = vec![card(4)];
        assert_eq!(session.state.legal_actions(1), vec![Action::Pickup]);
    }

    #[test]
    fn refill_prevents_early_finishing_and_never_recycles_burned_cards() {
        let mut game = playing_game(2);
        game.players[0] = PlayerCards {
            hand: vec![card(10)],
            face_up: vec![],
            face_down: vec![],
        };
        game.deck = Deck::new(vec![card(4), card(5)]);
        game.advance(0, &Action::Play { indices: vec![0] });
        assert_eq!(game.players[0].hand.len(), 2);
        assert!(game.deck.is_empty());
        assert_eq!(game.burned, 1);
        assert!(game.pile.is_empty());
        assert!(game.finished.is_empty());
        assert_eq!(game.turn.players, vec![0]);
        let before = game.players[0].hand.clone();
        game.refill_hand(0);
        assert_eq!(game.players[0].hand, before);
        assert!(game.deck.is_empty());
        assert_eq!(game.burned, 1);
    }

    #[test]
    fn illegal_zone_and_card_selections_leave_state_and_history_unchanged() {
        let mut session = playing_session(2);
        session.state.pile = vec![card(6)];
        session.state.players[0].hand = vec![card(8), card(9), card(4)];
        let before = serde_json::to_value(&session).unwrap();
        for action in [
            Action::Play { indices: vec![] },
            Action::Play {
                indices: vec![0, 1],
            },
            Action::Play { indices: vec![2] },
            Action::Play {
                indices: vec![usize::MAX],
            },
            Action::Blind { index: 0 },
        ] {
            assert_eq!(session.act(0, action).unwrap_err(), Error::IllegalMove);
            assert_eq!(serde_json::to_value(&session).unwrap(), before);
        }
        session.state.deck = Deck::new(vec![]);
        session.state.players[0] = PlayerCards {
            hand: vec![],
            face_up: vec![card(8)],
            face_down: vec![card(9)],
        };
        assert_eq!(
            session.state.legal_actions(0),
            vec![Action::Play { indices: vec![0] }, Action::Pickup]
        );
        session.act(0, Action::Play { indices: vec![0] }).unwrap();
        session.state.turn.players = vec![0];
        assert_eq!(
            session.state.legal_actions(0),
            vec![Action::Blind { index: 0 }, Action::Pickup]
        );
        let before = serde_json::to_value(&session).unwrap();
        for action in [
            Action::Play { indices: vec![0] },
            Action::Blind { index: usize::MAX },
        ] {
            assert_eq!(session.act(0, action).unwrap_err(), Error::IllegalMove);
            assert_eq!(serde_json::to_value(&session).unwrap(), before);
        }
    }

    #[test]
    fn successful_blind_play_can_finish_and_no_actions_follow_the_result() {
        let mut session = playing_session(2);
        session.state.deck = Deck::new(vec![]);
        session.state.players[0] = PlayerCards {
            hand: vec![],
            face_up: vec![],
            face_down: vec![card(10)],
        };
        session.state.pile = vec![card(14)];
        let record = session.act(0, Action::Blind { index: 0 }).unwrap();
        let outcome = Outcome {
            winners: vec![0],
            scores: None,
        };
        assert!(record.events.contains(&Event::Burned));
        assert_eq!(
            record.events.last(),
            Some(&Event::Finished {
                outcome: outcome.clone()
            })
        );
        assert_eq!(session.state.outcome(), Some(outcome));
        assert_eq!(session.state.finished, vec![0]);
        let before = serde_json::to_value(&session).unwrap();
        for player in 0..2 {
            assert!(session.state.legal_actions(player).is_empty());
            assert_eq!(
                session.act(player, Action::Pickup).unwrap_err(),
                Error::Finished
            );
            assert_eq!(serde_json::to_value(&session).unwrap(), before);
        }
    }

    #[test]
    fn finished_seats_are_skipped_and_winners_keep_finishing_order() {
        let mut session = playing_session(4);
        session.state.deck = Deck::new(vec![]);
        session.state.players[1] = PlayerCards {
            hand: vec![],
            face_up: vec![],
            face_down: vec![],
        };
        session.state.finished = vec![1];
        for player in [0, 2] {
            session.state.players[player] = PlayerCards {
                hand: vec![card(8)],
                face_up: vec![],
                face_down: vec![],
            };
        }
        session.act(0, Action::Play { indices: vec![0] }).unwrap();
        assert_eq!(session.state.finished, vec![1, 0]);
        assert_eq!(session.state.turn.players, vec![2]);
        assert!(session.state.outcome().is_none());
        let before = serde_json::to_value(&session).unwrap();
        for player in [0, 1, 3, usize::MAX] {
            assert!(session.state.legal_actions(player).is_empty());
            assert_eq!(
                session.act(player, Action::Pickup).unwrap_err(),
                Error::NotYourTurn
            );
            assert_eq!(serde_json::to_value(&session).unwrap(), before);
        }
        session.act(2, Action::Play { indices: vec![0] }).unwrap();
        assert_eq!(session.state.finished, vec![1, 0, 2]);
        assert_eq!(
            session.state.outcome().unwrap(),
            Outcome {
                winners: vec![1, 0, 2],
                scores: None
            }
        );
    }

    #[test]
    fn reset_burn_and_four_of_a_kind() {
        let mut game = playing_game(2);
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
        assert_eq!(game.players[0].hand.len(), 3);
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
        let mut game = playing_game(2);
        game.deck = Deck::new(vec![]);
        game.pile = vec![card(14)];
        game.players[0] = PlayerCards {
            hand: vec![],
            face_up: vec![],
            face_down: vec![card(4), card(5)],
        };
        assert!(game.legal_actions(0).contains(&Action::Blind { index: 0 }));
        game.advance(0, &Action::Blind { index: 0 });
        assert_eq!(game.players[0].hand, vec![card(14), card(4)]);
        assert_eq!(game.view(1).players[0].down, 1);
        assert_eq!(game.turn.players, vec![1]);
    }

    #[test]
    fn repeated_or_unsorted_indices_are_rejected() {
        let mut game = playing_session(2);
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
        let game = playing_game(2);
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
        let mut game = playing_game(3);
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
        assert_eq!(game.turn.number, 4);
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
            Action::SelectFaceUp { .. } => unreachable!("setup selects face-up cards separately"),
        }
    }

    fn is_empty(&self) -> bool {
        self.hand.is_empty() && self.face_up.is_empty() && self.face_down.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shithead {
    #[serde(deserialize_with = "crate::game::deserialize_rules_version::<_, 4>")]
    rules_version: u32,
    config: GameConfig,
    setup: bool,
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
    pub setup: bool,
    pub hand: Vec<Card>,
    pub players: Vec<PlayerView>,
    pub top: Option<Card>,
    pub effective_top: Option<Card>,
    pub pile: usize,
    pub stock: usize,
    pub burned: usize,
    pub finished: Vec<Player>,
}

impl Shithead {
    fn effective_top(&self) -> Option<Card> {
        self.pile.iter().rev().find(|card| card.rank != 3).copied()
    }

    fn playable(&self, card: Card) -> bool {
        if card.rank == RESET_RANK || card.rank == 3 {
            return true;
        }
        self.effective_top().is_none_or(|top| match top.rank {
            RESET_RANK => true,
            7 => card.rank <= 7,
            _ => card.rank == BURN_RANK || card.rank >= top.rank,
        })
    }

    fn same_rank_plays(&self, source: &[Card]) -> Vec<Action> {
        let mut actions = vec![];
        for rank in 2..=14 {
            let indices = source
                .iter()
                .enumerate()
                .filter(|(_, card)| card.rank == rank)
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
                    .filter(|indices| {
                        self.playable(source[indices[0]])
                            || (self.config.four_card_bombs
                                && indices.len() == MATCHING_CARDS_TO_BURN)
                    })
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
        let playable = self.playable(played[0])
            || (self.config.four_card_bombs && played.len() == MATCHING_CARDS_TO_BURN);
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
    type Config = GameConfig;
    type Action = Action;
    type Event = Event;
    type View = View;
    fn with_config(players: usize, seed: Seed, config: Self::Config) -> Result<Self, Error> {
        if !(2..=5).contains(&players) {
            return Err(Error::PlayerCount);
        }
        let mut deck = standard_deck();
        deck.shuffle(&mut ChaCha12Rng::from_seed(seed));
        let turn = Turn {
            players: (0..players).collect(),
            number: 0,
        };
        let players = (0..players)
            .map(|_| PlayerCards {
                face_down: (0..HAND_SIZE).map(|_| deck.draw().unwrap()).collect(),
                face_up: vec![],
                hand: (0..INITIAL_HAND_SIZE)
                    .map(|_| deck.draw().unwrap())
                    .collect(),
            })
            .collect();
        Ok(Self {
            rules_version: 4,
            config,
            setup: true,
            players,
            deck,
            pile: vec![],
            burned: 0,
            finished: vec![],
            turn,
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
        if self.setup {
            let mut actions = vec![];
            for first in 0..cards.hand.len() {
                for second in first + 1..cards.hand.len() {
                    for third in second + 1..cards.hand.len() {
                        actions.push(Action::SelectFaceUp {
                            indices: vec![first, second, third],
                        });
                    }
                }
            }
            return actions;
        }
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
        if let Action::SelectFaceUp { indices } = action {
            self.players[player].face_up = indices
                .iter()
                .map(|&index| self.players[player].hand[index])
                .collect();
            for &index in indices.iter().rev() {
                self.players[player].hand.remove(index);
            }
            self.turn.players.retain(|&seat| seat != player);
            self.turn.number += 1;
            if self.turn.players.is_empty() {
                self.setup = false;
                self.turn.players = vec![0];
            }
            return events;
        }
        let mut extra_turn = false;
        match action {
            Action::SelectFaceUp { .. } => unreachable!("setup handled above"),
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
            setup: self.setup,
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
            effective_top: self.effective_top(),
            pile: self.pile.len(),
            stock: self.deck.len(),
            burned: self.burned,
            finished: self.finished.clone(),
        }
    }
}
