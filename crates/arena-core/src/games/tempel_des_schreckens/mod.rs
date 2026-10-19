use crate::{
    cards::Deck,
    game::{Error, Event, Game, Outcome, Player, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const ROUNDS: usize = 4;
const INITIAL_HAND_SIZE: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as = TempelCard)]
pub enum Card {
    Gold,
    Trap,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as = TempelRole)]
pub enum Role {
    Adventurer,
    Guardian,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(as = TempelCounts)]
pub struct Counts {
    pub gold: usize,
    pub traps: usize,
    pub empty: usize,
}

impl Counts {
    fn add(&mut self, card: Card) {
        match card {
            Card::Gold => self.gold += 1,
            Card::Trap => self.traps += 1,
            Card::Empty => self.empty += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[schema(as = TempelDistribution)]
pub struct Distribution {
    pub adventurers: usize,
    pub guardians: usize,
    pub cards: Counts,
}

fn distribution(players: usize) -> Result<Distribution, Error> {
    let (adventurers, guardians, empty, gold, traps) = match players {
        3 => (2, 2, 8, 5, 2),
        4 => (3, 2, 12, 6, 2),
        5 => (3, 2, 16, 7, 2),
        6 => (4, 2, 20, 8, 2),
        7 => (5, 3, 26, 7, 2),
        8 => (6, 3, 30, 8, 2),
        9 => (6, 3, 34, 9, 2),
        10 => (7, 4, 37, 10, 3),
        _ => return Err(Error::PlayerCount),
    };
    Ok(Distribution {
        adventurers,
        guardians,
        cards: Counts { empty, gold, traps },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as = TempelPhase)]
pub enum Phase {
    ChooseKey,
    Inspect,
    Reveal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schema(as = TempelAction)]
pub enum Action {
    ChooseKey { player: Player },
    Ready,
    Reveal { player: Player, index: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Slot {
    card: Card,
    revealed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempelDesSchreckens {
    roles: Vec<Role>,
    hands: Vec<Vec<Slot>>,
    ready: Vec<bool>,
    phase: Phase,
    key: Option<Player>,
    round: usize,
    round_reveals: usize,
    revealed: Counts,
    rng: ChaCha12Rng,
    turn_number: u64,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = TempelView)]
pub struct View {
    pub role: Role,
    /// Counts of this viewer's unrevealed cards, without their positions.
    pub hand_counts: Counts,
    /// Only face-up cards have values, including in the viewer's own row.
    pub cards: Vec<Vec<Option<Card>>>,
    pub ready: Vec<bool>,
    pub phase: Phase,
    pub key: Option<Player>,
    pub round: usize,
    pub round_reveals: usize,
    pub revealed: Counts,
    pub distribution: Distribution,
}

impl TempelDesSchreckens {
    fn deal(&mut self, mut deck: Deck<Card>) {
        deck.shuffle(&mut self.rng);
        self.hands = vec![vec![]; self.roles.len()];
        let mut seat = 0;
        while let Some(card) = deck.draw() {
            self.hands[seat % self.roles.len()].push(Slot {
                card,
                revealed: false,
            });
            seat += 1;
        }
        self.ready.fill(false);
        self.round_reveals = 0;
    }

    fn finish(&mut self, winning_role: Role) {
        self.outcome = Some(Outcome {
            winners: self
                .roles
                .iter()
                .enumerate()
                .filter_map(|(player, &role)| (role == winning_role).then_some(player))
                .collect(),
            scores: None,
        });
    }
}

impl Game for TempelDesSchreckens {
    type Config = crate::game::GameConfig;
    type Action = Action;
    type Event = Event;
    type View = View;

    fn with_config(players: usize, seed: Seed, _: Self::Config) -> Result<Self, Error> {
        let setup = distribution(players)?;
        let mut rng = ChaCha12Rng::from_seed(seed);
        let mut role_deck = Deck::new(
            std::iter::repeat_n(Role::Adventurer, setup.adventurers)
                .chain(std::iter::repeat_n(Role::Guardian, setup.guardians))
                .collect(),
        );
        role_deck.shuffle(&mut rng);
        let roles = (0..players).map(|_| role_deck.draw().unwrap()).collect();
        let deck = Deck::new(
            std::iter::repeat_n(Card::Gold, setup.cards.gold)
                .chain(std::iter::repeat_n(Card::Trap, setup.cards.traps))
                .chain(std::iter::repeat_n(Card::Empty, setup.cards.empty))
                .collect(),
        );
        let mut game = Self {
            roles,
            hands: vec![],
            ready: vec![false; players],
            phase: Phase::ChooseKey,
            key: None,
            round: 1,
            round_reveals: 0,
            revealed: Counts::default(),
            rng,
            turn_number: 0,
            outcome: None,
        };
        game.deal(deck);
        debug_assert!(
            game.hands
                .iter()
                .all(|hand| hand.len() == INITIAL_HAND_SIZE)
        );
        Ok(game)
    }

    fn turn(&self) -> Turn {
        let players = if self.outcome.is_some() {
            vec![]
        } else {
            match self.phase {
                Phase::ChooseKey => vec![0],
                Phase::Inspect => (0..self.roles.len()).filter(|&p| !self.ready[p]).collect(),
                Phase::Reveal => vec![self.key.unwrap()],
            }
        };
        Turn {
            players,
            number: self.turn_number,
        }
    }
    fn outcome(&self) -> Option<Outcome> {
        self.outcome.clone()
    }
    fn legal_actions(&self, player: Player) -> Vec<Action> {
        if !self.turn().players.contains(&player) {
            return vec![];
        }
        match self.phase {
            Phase::ChooseKey => (0..self.roles.len())
                .map(|player| Action::ChooseKey { player })
                .collect(),
            Phase::Inspect => vec![Action::Ready],
            Phase::Reveal => self
                .hands
                .iter()
                .enumerate()
                .filter(|(target, _)| *target != player)
                .flat_map(|(player, hand)| {
                    hand.iter()
                        .enumerate()
                        .filter(|(_, slot)| !slot.revealed)
                        .map(move |(index, _)| Action::Reveal { player, index })
                })
                .collect(),
        }
    }
    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        match *action {
            Action::ChooseKey { player } => {
                self.key = Some(player);
                self.phase = Phase::Inspect;
            }
            Action::Ready => {
                self.ready[player] = true;
                if self.ready.iter().all(|&ready| ready) {
                    self.phase = Phase::Reveal;
                }
            }
            Action::Reveal {
                player: target,
                index,
            } => {
                let slot = &mut self.hands[target][index];
                slot.revealed = true;
                self.revealed.add(slot.card);
                self.round_reveals += 1;
                self.key = Some(target);
                let totals = distribution(self.roles.len()).unwrap().cards;
                if self.revealed.gold == totals.gold {
                    self.finish(Role::Adventurer);
                } else if self.revealed.traps == totals.traps
                    || (self.round == ROUNDS && self.round_reveals == self.roles.len())
                {
                    self.finish(Role::Guardian);
                } else if self.round_reveals == self.roles.len() {
                    let deck = Deck::new(
                        self.hands
                            .iter()
                            .flatten()
                            .filter(|slot| !slot.revealed)
                            .map(|slot| slot.card)
                            .collect(),
                    );
                    self.round += 1;
                    self.deal(deck);
                    self.phase = Phase::Inspect;
                }
            }
        }
        self.turn_number += 1;
        vec![Event::Played { player }]
    }
    fn view(&self, viewer: Player) -> View {
        let mut hand_counts = Counts::default();
        for slot in self.hands[viewer].iter().filter(|slot| !slot.revealed) {
            hand_counts.add(slot.card);
        }
        View {
            role: self.roles[viewer],
            hand_counts,
            cards: self
                .hands
                .iter()
                .map(|hand| {
                    hand.iter()
                        .map(|slot| slot.revealed.then_some(slot.card))
                        .collect()
                })
                .collect(),
            ready: self.ready.clone(),
            phase: self.phase,
            key: self.key,
            round: self.round,
            round_reveals: self.round_reveals,
            revealed: self.revealed,
            distribution: distribution(self.roles.len()).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Session;

    fn session(players: usize) -> Session<TempelDesSchreckens> {
        Session::new(players, [0; 32]).unwrap()
    }
    fn ready_all(s: &mut Session<TempelDesSchreckens>) {
        for player in (0..s.state.roles.len()).rev() {
            s.act(player, Action::Ready).unwrap();
        }
    }
    fn reject(s: &mut Session<TempelDesSchreckens>, player: Player, action: Action) {
        let before = serde_json::to_value(&*s).unwrap();
        assert!(s.act(player, action).is_err());
        assert_eq!(serde_json::to_value(&*s).unwrap(), before);
    }

    #[test]
    fn setup_matches_every_distribution_and_never_exposes_hidden_positions_or_other_roles() {
        for (players, adventurers, guardians, empty, gold, traps) in [
            (3, 2, 2, 8, 5, 2),
            (4, 3, 2, 12, 6, 2),
            (5, 3, 2, 16, 7, 2),
            (6, 4, 2, 20, 8, 2),
            (7, 5, 3, 26, 7, 2),
            (8, 6, 3, 30, 8, 2),
            (9, 6, 3, 34, 9, 2),
            (10, 7, 4, 37, 10, 3),
        ] {
            let s = session(players);
            assert_eq!(
                serde_json::to_value(&s).unwrap(),
                serde_json::to_value(session(players)).unwrap()
            );
            assert!(
                s.state
                    .hands
                    .iter()
                    .all(|hand| hand.len() == 5 && hand.iter().all(|slot| !slot.revealed))
            );
            let mut counts = Counts::default();
            for slot in s.state.hands.iter().flatten() {
                counts.add(slot.card);
            }
            assert_eq!(counts, Counts { empty, gold, traps });
            let actual_a = s
                .state
                .roles
                .iter()
                .filter(|&&r| r == Role::Adventurer)
                .count();
            let actual_g = players - actual_a;
            assert!(actual_a <= adventurers && actual_g <= guardians);
            assert_eq!(
                (adventurers - actual_a) + (guardians - actual_g),
                adventurers + guardians - players
            );
            for viewer in 0..players {
                let view = s.state.view(viewer);
                assert_eq!(view.role, s.state.roles[viewer]);
                assert_eq!(
                    view.hand_counts.gold + view.hand_counts.traps + view.hand_counts.empty,
                    5
                );
                assert!(view.cards.iter().flatten().all(Option::is_none));
                let value = serde_json::to_value(&view).unwrap();
                for private in ["roles", "hands", "rng", "seed", "unused_role"] {
                    assert!(value.get(private).is_none());
                }
            }
            assert_eq!(s.state.turn().players, vec![0]);
            assert_eq!(s.state.phase, Phase::ChooseKey);
        }
        for players in [0, 1, 2, 11, usize::MAX] {
            assert!(matches!(
                TempelDesSchreckens::new(players, [0; 32]),
                Err(Error::PlayerCount)
            ));
        }
    }

    #[test]
    fn unused_role_can_belong_to_either_team() {
        for players in [3, 4, 7, 8, 10] {
            let setup = distribution(players).unwrap();
            let mut omitted = [false; 2];
            for seed in 0..64 {
                let game = TempelDesSchreckens::new(players, [seed; 32]).unwrap();
                let count = game
                    .roles
                    .iter()
                    .filter(|&&r| r == Role::Adventurer)
                    .count();
                omitted[usize::from(count == setup.adventurers)] = true;
            }
            assert_eq!(omitted, [true, true], "{players} players");
        }
    }

    #[test]
    fn host_chooses_any_initial_key_and_readiness_is_simultaneous() {
        for key in 0..3 {
            let mut s = session(3);
            reject(&mut s, 1, Action::ChooseKey { player: key });
            reject(&mut s, 0, Action::ChooseKey { player: usize::MAX });
            reject(&mut s, 0, Action::Ready);
            s.act(0, Action::ChooseKey { player: key }).unwrap();
            assert_eq!(s.state.key, Some(key));
            assert_eq!(s.state.turn().players, vec![0, 1, 2]);
            reject(&mut s, 0, Action::ChooseKey { player: 0 });
            reject(
                &mut s,
                key,
                Action::Reveal {
                    player: (key + 1) % 3,
                    index: 0,
                },
            );
            for player in [2, 0, 1] {
                s.act(player, Action::Ready).unwrap();
                if player != 1 {
                    reject(&mut s, player, Action::Ready);
                }
            }
            assert_eq!(s.state.phase, Phase::Reveal);
            assert_eq!(s.state.turn().players, vec![key]);
            reject(&mut s, key, Action::Ready);
            reject(&mut s, usize::MAX, Action::Ready);
        }
    }

    #[test]
    fn reveal_moves_the_key_updates_only_public_piles_and_preserves_slot_indices() {
        let mut s = session(3);
        s.act(0, Action::ChooseKey { player: 0 }).unwrap();
        ready_all(&mut s);
        reject(
            &mut s,
            1,
            Action::Reveal {
                player: 2,
                index: 0,
            },
        );
        reject(
            &mut s,
            0,
            Action::Reveal {
                player: 0,
                index: 0,
            },
        );
        reject(
            &mut s,
            0,
            Action::Reveal {
                player: 3,
                index: 0,
            },
        );
        reject(
            &mut s,
            0,
            Action::Reveal {
                player: 1,
                index: usize::MAX,
            },
        );
        let card = s.state.hands[1][2].card;
        let own_counts = s.state.view(1).hand_counts;
        s.act(
            0,
            Action::Reveal {
                player: 1,
                index: 2,
            },
        )
        .unwrap();
        let mut expected = Counts::default();
        expected.add(card);
        assert_eq!(s.state.revealed, expected);
        assert_eq!(s.state.key, Some(1));
        assert_eq!(s.state.hands[1].len(), 5);
        assert_eq!(s.state.view(0).cards[1][2], Some(card));
        assert_eq!(s.state.view(2).cards[1][2], Some(card));
        let remaining = s.state.view(1).hand_counts;
        assert_eq!(remaining.gold + expected.gold, own_counts.gold);
        assert_eq!(remaining.traps + expected.traps, own_counts.traps);
        assert_eq!(remaining.empty + expected.empty, own_counts.empty);
        s.act(
            1,
            Action::Reveal {
                player: 0,
                index: 0,
            },
        )
        .unwrap();
        reject(
            &mut s,
            0,
            Action::Reveal {
                player: 1,
                index: 2,
            },
        );
    }

    #[test]
    fn four_rounds_shrink_rows_keep_key_and_roles_and_expire_after_exactly_n_reveals() {
        let mut s = session(3);
        for slot in s.state.hands.iter_mut().flatten() {
            slot.card = Card::Empty;
        }
        let roles = s.state.roles.clone();
        s.act(0, Action::ChooseKey { player: 2 }).unwrap();
        for round in 1..=4 {
            assert_eq!(s.state.round, round);
            assert_eq!(s.state.phase, Phase::Inspect);
            assert!(s.state.hands.iter().all(|hand| hand.len() == 6 - round));
            assert!(s.state.ready.iter().all(|&r| !r));
            let key = s.state.key;
            ready_all(&mut s);
            assert_eq!(s.state.key, key);
            // Alternate between two rows; the third row remains entirely hidden.
            let mut last_target = 0;
            for _ in 0..3 {
                let actor = s.state.key.unwrap();
                let player = if actor == 0 { 1 } else { 0 };
                let index = s.state.hands[player]
                    .iter()
                    .position(|slot| !slot.revealed)
                    .unwrap();
                s.act(actor, Action::Reveal { player, index }).unwrap();
                last_target = player;
            }
            assert_eq!(s.state.key, Some(last_target));
            assert_eq!(s.state.roles, roles);
            assert_eq!(s.state.revealed.empty, round * 3);
            if round != 4 {
                assert!(s.state.outcome.is_none());
                assert!(s.state.hands.iter().flatten().all(|slot| !slot.revealed));
                assert_eq!(s.state.round_reveals, 0);
            } else {
                assert_eq!(s.state.round_reveals, 3);
                assert!(s.state.outcome.is_some());
                assert_eq!(
                    s.state.outcome.as_ref().unwrap().winners,
                    roles
                        .iter()
                        .enumerate()
                        .filter_map(|(p, &r)| (r == Role::Guardian).then_some(p))
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn all_gold_wins_even_on_the_final_reveal_and_results_freeze_state() {
        for final_reveal in [false, true] {
            let mut s = session(3);
            s.act(0, Action::ChooseKey { player: 0 }).unwrap();
            ready_all(&mut s);
            s.state.revealed.gold = 4;
            s.state.hands[1][0].card = Card::Gold;
            if final_reveal {
                s.state.round = 4;
                s.state.round_reveals = 2;
            }
            let roles = s.state.roles.clone();
            let record = s
                .act(
                    0,
                    Action::Reveal {
                        player: 1,
                        index: 0,
                    },
                )
                .unwrap();
            assert!(matches!(record.events.last(), Some(Event::Finished { .. })));
            assert_eq!(
                s.state.outcome.as_ref().unwrap().winners,
                roles
                    .iter()
                    .enumerate()
                    .filter_map(|(p, &r)| (r == Role::Adventurer).then_some(p))
                    .collect::<Vec<_>>()
            );
            assert!(s.state.outcome.as_ref().unwrap().scores.is_none());
            assert_eq!(s.state.key, Some(1));
            assert!(s.state.turn().players.is_empty());
            reject(
                &mut s,
                1,
                Action::Reveal {
                    player: 0,
                    index: 0,
                },
            );
            reject(&mut s, 0, Action::ChooseKey { player: 0 });
            assert!(s.state.view(0).cards[0].iter().all(Option::is_none));
        }
    }

    #[test]
    fn guardians_need_every_trap_including_three_with_ten_players() {
        for players in [3, 10] {
            let mut s = session(players);
            s.act(0, Action::ChooseKey { player: 0 }).unwrap();
            ready_all(&mut s);
            let traps = distribution(players).unwrap().cards.traps;
            s.state.hands[1][0].card = Card::Trap;
            s.state.hands[0][0].card = Card::Trap;
            s.state.revealed.traps = traps - 2;
            s.act(
                0,
                Action::Reveal {
                    player: 1,
                    index: 0,
                },
            )
            .unwrap();
            assert!(s.state.outcome.is_none());
            s.act(
                1,
                Action::Reveal {
                    player: 0,
                    index: 0,
                },
            )
            .unwrap();
            assert_eq!(s.state.revealed.traps, traps);
            assert_eq!(
                s.state.outcome.as_ref().unwrap().winners,
                s.state
                    .roles
                    .iter()
                    .enumerate()
                    .filter_map(|(p, &r)| (r == Role::Guardian).then_some(p))
                    .collect::<Vec<_>>()
            );
            reject(
                &mut s,
                0,
                Action::Reveal {
                    player: 1,
                    index: 1,
                },
            );
        }
    }
}
