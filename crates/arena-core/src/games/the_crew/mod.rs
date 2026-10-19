use crate::{
    cards::Deck,
    game::{Error, Event, Game, Outcome, Player, Resolution, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Printed colors; black is the trump color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as = CrewColor)]
pub enum Color {
    Pink,
    Yellow,
    Blue,
    Green,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[schema(as = CrewCard)]
pub struct Card {
    pub color: Color,
    pub number: u8,
}

/// A statement about a card relative to its owner's current hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as = CrewDeclaration)]
pub enum Declaration {
    Lowest,
    Only,
    Highest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(as = CrewCommunication)]
pub struct Communication {
    pub card: Card,
    pub declaration: Declaration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schema(as = CrewAction)]
pub enum Action {
    Play {
        index: usize,
    },
    Communicate {
        index: usize,
        declaration: Declaration,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = CrewPlay)]
pub struct Play {
    pub player: Player,
    pub card: Card,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = CrewTrick)]
pub struct Trick {
    pub cards: Vec<Play>,
    pub winner: Player,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheCrew {
    hands: Vec<Vec<Card>>,
    communications: Vec<Option<Communication>>,
    trick: Vec<Play>,
    completed: Vec<Trick>,
    next_player: Player,
    turn_number: u64,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = CrewView)]
pub struct View {
    pub hand: Vec<Card>,
    pub counts: Vec<usize>,
    pub communications: Vec<Option<Communication>>,
    pub trick: Vec<Play>,
    pub last_trick: Option<Trick>,
    pub tricks_won: Vec<usize>,
    pub next_player: Player,
    pub awaiting_result: bool,
}

impl TheCrew {
    fn awaiting_result(&self) -> bool {
        self.trick.is_empty() && self.hands.iter().any(Vec::is_empty)
    }

    fn declarations(&self, player: Player, card: Card) -> Vec<Declaration> {
        if card.color == Color::Black {
            return vec![];
        }
        let same: Vec<_> = self.hands[player]
            .iter()
            .filter(|c| c.color == card.color)
            .collect();
        let mut result = vec![];
        if same.iter().all(|c| c.number >= card.number) {
            result.push(Declaration::Lowest);
        }
        if same.len() == 1 {
            result.push(Declaration::Only);
        }
        if same.iter().all(|c| c.number <= card.number) {
            result.push(Declaration::Highest);
        }
        result
    }
}

impl Game for TheCrew {
    type Config = crate::game::GameConfig;
    type Action = Action;
    type Event = Event;
    type View = View;

    fn with_config(players: usize, seed: Seed, _: Self::Config) -> Result<Self, Error> {
        if !(3..=5).contains(&players) {
            return Err(Error::PlayerCount);
        }
        let mut deck = Deck::new(
            [
                Color::Pink,
                Color::Yellow,
                Color::Blue,
                Color::Green,
                Color::Black,
            ]
            .into_iter()
            .flat_map(|color| {
                (1..=if color == Color::Black { 4 } else { 9 })
                    .map(move |number| Card { color, number })
            })
            .collect(),
        );
        deck.shuffle(&mut ChaCha12Rng::from_seed(seed));
        let mut hands = vec![vec![]; players];
        let mut seat = 0;
        while let Some(card) = deck.draw() {
            hands[seat % players].push(card);
            seat += 1;
        }
        for hand in &mut hands {
            hand.sort();
        }
        let next_player = hands
            .iter()
            .position(|hand| {
                hand.contains(&Card {
                    color: Color::Black,
                    number: 4,
                })
            })
            .unwrap();
        Ok(Self {
            hands,
            communications: vec![None; players],
            trick: vec![],
            completed: vec![],
            next_player,
            turn_number: 0,
            outcome: None,
        })
    }

    fn turn(&self) -> Turn {
        Turn {
            players: (0..self.hands.len())
                .filter(|&p| !self.legal_actions(p).is_empty())
                .collect(),
            number: self.turn_number,
        }
    }
    fn outcome(&self) -> Option<Outcome> {
        self.outcome.clone()
    }

    fn legal_actions(&self, player: Player) -> Vec<Action> {
        if self.outcome.is_some() || player >= self.hands.len() {
            return vec![];
        }
        let mut actions = vec![];
        if !self.awaiting_result() {
            if player == self.next_player {
                let lead = self.trick.first().map(|p| p.card.color);
                let must_follow = self.hands[player].iter().any(|c| Some(c.color) == lead);
                actions.extend(
                    self.hands[player]
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| !must_follow || Some(c.color) == lead)
                        .map(|(index, _)| Action::Play { index }),
                );
            }
            if self.trick.is_empty() && self.communications[player].is_none() {
                for (index, &card) in self.hands[player].iter().enumerate() {
                    actions.extend(
                        self.declarations(player, card)
                            .into_iter()
                            .map(|declaration| Action::Communicate { index, declaration }),
                    );
                }
            }
        }
        actions
    }

    fn resolutions(&self) -> Vec<Resolution> {
        if self.outcome.is_some() {
            vec![]
        } else {
            vec![
                Resolution::SharedResult { won: true },
                Resolution::SharedResult { won: false },
            ]
        }
    }

    fn resolve(&mut self, resolution: Resolution) -> Vec<Event> {
        let Resolution::SharedResult { won } = resolution;
        self.outcome = Some(Outcome {
            winners: if won {
                (0..self.hands.len()).collect()
            } else {
                vec![]
            },
            scores: None,
        });
        vec![]
    }

    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        let mut events = vec![Event::Played { player }];
        match *action {
            Action::Communicate { index, declaration } => {
                self.communications[player] = Some(Communication {
                    card: self.hands[player][index],
                    declaration,
                })
            }
            Action::Play { index } => {
                let card = self.hands[player].remove(index);
                self.trick.push(Play { player, card });
                self.next_player = (player + 1) % self.hands.len();
                if self.trick.len() == self.hands.len() {
                    let lead = self.trick[0].card.color;
                    let winner = self
                        .trick
                        .iter()
                        .max_by_key(|p| {
                            (
                                if p.card.color == Color::Black {
                                    2
                                } else if p.card.color == lead {
                                    1
                                } else {
                                    0
                                },
                                p.card.number,
                            )
                        })
                        .unwrap()
                        .player;
                    self.completed.push(Trick {
                        cards: std::mem::take(&mut self.trick),
                        winner,
                    });
                    self.next_player = winner;
                    events.push(Event::Captured {
                        player: winner,
                        count: self.hands.len(),
                    });
                }
            }
        }
        self.turn_number += 1;
        events
    }

    fn view(&self, viewer: Player) -> View {
        View {
            hand: self.hands[viewer].clone(),
            counts: self.hands.iter().map(Vec::len).collect(),
            communications: self.communications.clone(),
            trick: self.trick.clone(),
            last_trick: self.completed.last().cloned(),
            tricks_won: (0..self.hands.len())
                .map(|p| self.completed.iter().filter(|t| t.winner == p).count())
                .collect(),
            next_player: self.next_player,
            awaiting_result: self.awaiting_result(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Session;

    fn card(color: Color, number: u8) -> Card {
        Card { color, number }
    }
    fn fixture(hands: Vec<Vec<Card>>) -> Session<TheCrew> {
        let mut session = Session::<TheCrew>::new(hands.len(), [0; 32]).unwrap();
        session.state.hands = hands;
        session.state.next_player = 0;
        session
    }
    fn reject(session: &mut Session<TheCrew>, player: Player, action: Action) {
        let before = serde_json::to_value(&*session).unwrap();
        assert!(session.act(player, action).is_err());
        assert_eq!(serde_json::to_value(&*session).unwrap(), before);
    }

    #[test]
    fn deal_is_seeded_complete_unique_and_black_four_holder_leads() {
        for players in 3..=5 {
            for seed in 0..16 {
                let game = TheCrew::new(players, [seed; 32]).unwrap();
                assert_eq!(
                    serde_json::to_value(&game).unwrap(),
                    serde_json::to_value(TheCrew::new(players, [seed; 32]).unwrap()).unwrap()
                );
                let mut cards: Vec<_> = game.hands.iter().flatten().copied().collect();
                assert_eq!(cards.len(), 40);
                cards.sort();
                cards.dedup();
                assert_eq!(cards.len(), 40);
                for color in [
                    Color::Pink,
                    Color::Yellow,
                    Color::Blue,
                    Color::Green,
                    Color::Black,
                ] {
                    assert_eq!(
                        cards
                            .iter()
                            .filter(|c| c.color == color)
                            .map(|c| c.number)
                            .collect::<Vec<_>>(),
                        (1..=if color == Color::Black { 4 } else { 9 }).collect::<Vec<_>>()
                    );
                }
                assert_eq!(
                    game.hands.iter().map(Vec::len).collect::<Vec<_>>(),
                    match players {
                        3 => vec![14, 13, 13],
                        4 => vec![10; 4],
                        _ => vec![8; 5],
                    }
                );
                assert!(game.hands[game.next_player].contains(&card(Color::Black, 4)));
                assert_eq!(
                    game.legal_actions(game.next_player)
                        .iter()
                        .filter(|a| matches!(a, Action::Play { .. }))
                        .count(),
                    game.hands[game.next_player].len()
                );
                for viewer in 0..players {
                    let view = game.view(viewer);
                    assert_eq!(view.hand, game.hands[viewer]);
                    let json = serde_json::to_value(view).unwrap();
                    for key in ["hands", "seed", "deck", "completed"] {
                        assert!(json.get(key).is_none());
                    }
                }
            }
        }
        for players in [0, 1, 2, 6, usize::MAX] {
            assert!(matches!(
                TheCrew::new(players, [0; 32]),
                Err(Error::PlayerCount)
            ));
        }
    }

    #[test]
    fn following_color_is_mandatory_including_trumps_and_void_players_can_discard() {
        for color in [
            Color::Pink,
            Color::Yellow,
            Color::Blue,
            Color::Green,
            Color::Black,
        ] {
            let other = if color == Color::Pink {
                Color::Green
            } else {
                Color::Pink
            };
            let mut s = fixture(vec![
                vec![card(color, 1)],
                vec![card(other, 9), card(color, 2)],
                vec![card(other, 8)],
            ]);
            s.act(0, Action::Play { index: 0 }).unwrap();
            assert_eq!(s.state.legal_actions(1), vec![Action::Play { index: 1 }]);
            reject(&mut s, 1, Action::Play { index: 0 });
            reject(&mut s, 2, Action::Play { index: 0 });
            reject(&mut s, 1, Action::Play { index: usize::MAX });
            s.act(1, Action::Play { index: 1 }).unwrap();
            assert_eq!(s.state.legal_actions(2), vec![Action::Play { index: 0 }]);
            s.act(2, Action::Play { index: 0 }).unwrap();
            assert_eq!(s.state.completed[0].winner, 1);
        }
    }

    #[test]
    fn highest_trump_or_highest_led_color_wins_and_leads_next_in_seat_order() {
        use Color::*;
        for (played, winner) in [
            (vec![card(Pink, 4), card(Blue, 9), card(Pink, 6)], 2),
            (vec![card(Pink, 9), card(Black, 1), card(Black, 3)], 2),
            (vec![card(Black, 4), card(Black, 2), card(Pink, 9)], 0),
        ] {
            let mut s = fixture(
                played
                    .into_iter()
                    .map(|c| vec![c, card(Green, 1)])
                    .collect(),
            );
            for player in 0..3 {
                s.act(player, Action::Play { index: 0 }).unwrap();
            }
            assert_eq!(s.state.next_player, winner);
            assert_eq!(s.state.completed[0].winner, winner);
            assert_eq!(s.state.completed[0].cards.len(), 3);
            assert!(s.state.trick.is_empty());
            assert!(
                s.state
                    .legal_actions(winner)
                    .contains(&Action::Play { index: 0 })
            );
            s.act(winner, Action::Play { index: 0 }).unwrap();
            assert_eq!(s.state.next_player, (winner + 1) % 3);
        }
    }

    #[test]
    fn communication_only_allows_truthful_extremes_and_singletons_allow_all_declarations() {
        let mut s = fixture(vec![
            vec![card(Color::Black, 4)],
            vec![
                card(Color::Pink, 1),
                card(Color::Pink, 5),
                card(Color::Pink, 9),
                card(Color::Blue, 3),
                card(Color::Black, 1),
            ],
            vec![card(Color::Green, 1)],
        ]);
        let comms: Vec<_> = s
            .state
            .legal_actions(1)
            .into_iter()
            .filter(|a| matches!(a, Action::Communicate { .. }))
            .collect();
        assert_eq!(
            comms,
            vec![
                Action::Communicate {
                    index: 0,
                    declaration: Declaration::Lowest
                },
                Action::Communicate {
                    index: 2,
                    declaration: Declaration::Highest
                },
                Action::Communicate {
                    index: 3,
                    declaration: Declaration::Lowest
                },
                Action::Communicate {
                    index: 3,
                    declaration: Declaration::Only
                },
                Action::Communicate {
                    index: 3,
                    declaration: Declaration::Highest
                }
            ]
        );
        for (index, declaration) in [
            (1, Declaration::Lowest),
            (1, Declaration::Highest),
            (0, Declaration::Only),
            (4, Declaration::Highest),
            (usize::MAX, Declaration::Only),
        ] {
            reject(&mut s, 1, Action::Communicate { index, declaration });
        }
        let hand = s.state.hands[1].clone();
        s.act(
            1,
            Action::Communicate {
                index: 2,
                declaration: Declaration::Highest,
            },
        )
        .unwrap();
        assert_eq!(s.state.hands[1], hand);
        assert_eq!(s.state.next_player, 0);
        assert_eq!(
            s.state.view(2).communications[1],
            Some(Communication {
                card: card(Color::Pink, 9),
                declaration: Declaration::Highest
            })
        );
        reject(
            &mut s,
            1,
            Action::Communicate {
                index: 0,
                declaration: Declaration::Lowest,
            },
        );
        s.act(0, Action::Play { index: 0 }).unwrap();
        reject(
            &mut s,
            2,
            Action::Communicate {
                index: 0,
                declaration: Declaration::Only,
            },
        );
    }

    #[test]
    fn communication_reopens_between_tricks_and_survives_the_communicated_card_play() {
        let mut s = fixture(vec![
            vec![card(Color::Pink, 1), card(Color::Blue, 1)],
            vec![card(Color::Pink, 2), card(Color::Blue, 2)],
            vec![card(Color::Pink, 3), card(Color::Blue, 3)],
        ]);
        s.act(
            2,
            Action::Communicate {
                index: 0,
                declaration: Declaration::Only,
            },
        )
        .unwrap();
        for player in 0..3 {
            s.act(player, Action::Play { index: 0 }).unwrap();
        }
        assert_eq!(
            s.state.communications[2].as_ref().unwrap().card,
            card(Color::Pink, 3)
        );
        assert!(
            !s.state
                .legal_actions(2)
                .iter()
                .any(|a| matches!(a, Action::Communicate { .. }))
        );
        s.act(
            1,
            Action::Communicate {
                index: 0,
                declaration: Declaration::Highest,
            },
        )
        .unwrap();
        assert_eq!(s.state.next_player, 2);
    }

    #[test]
    fn external_results_have_no_player_and_do_not_change_player_history() {
        for won in [false, true] {
            for partial in [false, true] {
                let mut s = Session::<TheCrew>::new(3, [2; 32]).unwrap();
                if partial {
                    let player = s.state.next_player;
                    s.act(player, Action::Play { index: 0 }).unwrap();
                }
                let actions = s.records().to_vec();
                let turn_number = s.state.turn().number;
                let record = s.resolve(Resolution::SharedResult { won }).unwrap();
                let outcome = Outcome {
                    winners: if won { vec![0, 1, 2] } else { vec![] },
                    scores: None,
                };
                assert_eq!(
                    record.events.last(),
                    Some(&Event::Finished {
                        outcome: outcome.clone()
                    })
                );
                assert_eq!(s.state.outcome(), Some(outcome));
                assert_eq!(s.records(), actions);
                assert_eq!(s.state.turn().number, turn_number);
                assert_eq!(s.resolution().unwrap().events.len(), 1);
                for player in 0..3 {
                    assert!(s.state.legal_actions(player).is_empty());
                }
                let before = serde_json::to_value(&s).unwrap();
                assert_eq!(
                    s.resolve(Resolution::SharedResult { won }).unwrap_err(),
                    Error::Finished
                );
                reject(&mut s, 0, Action::Play { index: 0 });
                assert_eq!(serde_json::to_value(&s).unwrap(), before);
            }
        }
    }
}
