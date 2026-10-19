use crate::game::{Error, Event, Game, Outcome, Player, Seed, Turn};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const PLAYERS: usize = 2;
const PITS_PER_PLAYER: usize = 6;
const SIDE_SIZE: usize = PITS_PER_PLAYER + 1;
const BOARD_SIZE: usize = PLAYERS * SIDE_SIZE;
const INITIAL_STONES: u16 = 4;

#[derive(Debug, Clone, Copy)]
enum PitKind {
    House { opposite: usize },
    Store,
}

#[derive(Debug, Clone)]
struct Pit {
    owner: Player,
    kind: PitKind,
    stones: u16,
}

impl Pit {
    fn is_house(&self) -> bool {
        matches!(self.kind, PitKind::House { .. })
    }

    fn is_store_for(&self, player: Player) -> bool {
        self.owner == player && matches!(self.kind, PitKind::Store)
    }

    fn take_stones(&mut self) -> u16 {
        std::mem::take(&mut self.stones)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "[u16; BOARD_SIZE]", into = "[u16; BOARD_SIZE]")]
struct Board {
    pits: [Pit; BOARD_SIZE],
}

// Keep the original flat stone counts in saved games and player views.
impl From<[u16; BOARD_SIZE]> for Board {
    fn from(stones: [u16; BOARD_SIZE]) -> Self {
        Self {
            pits: std::array::from_fn(|index| Pit {
                owner: index / SIDE_SIZE,
                kind: if index % SIDE_SIZE == PITS_PER_PLAYER {
                    PitKind::Store
                } else {
                    // Houses mirror across the board, excluding the two stores.
                    PitKind::House {
                        opposite: BOARD_SIZE - PLAYERS - index,
                    }
                },
                stones: stones[index],
            }),
        }
    }
}

impl From<Board> for [u16; BOARD_SIZE] {
    fn from(board: Board) -> Self {
        board.stones()
    }
}

impl Board {
    fn new() -> Self {
        let mut board = Self::from([0; BOARD_SIZE]);
        for pit in &mut board.pits {
            if pit.is_house() {
                pit.stones = INITIAL_STONES;
            }
        }
        board
    }

    fn stones(&self) -> [u16; BOARD_SIZE] {
        std::array::from_fn(|index| self.pits[index].stones)
    }

    fn store_mut(&mut self, player: Player) -> &mut Pit {
        self.pits
            .iter_mut()
            .find(|pit| pit.is_store_for(player))
            .expect("every player has a store")
    }

    fn sow(&mut self, player: Player, start: usize) -> usize {
        let mut stones = self.pits[start].take_stones();
        let mut index = start;
        while stones > 0 {
            index = (index + 1) % self.pits.len();
            let pit = &mut self.pits[index];
            if !pit.is_house() && pit.owner != player {
                continue;
            }
            pit.stones += 1;
            stones -= 1;
        }
        index
    }

    fn capture(&mut self, player: Player, landing: usize) -> Option<u16> {
        let pit = &self.pits[landing];
        let PitKind::House { opposite } = pit.kind else {
            return None;
        };
        if pit.owner != player || pit.stones != 1 || self.pits[opposite].stones == 0 {
            return None;
        }
        let captured = self.pits[landing].take_stones() + self.pits[opposite].take_stones();
        self.store_mut(player).stones += captured;
        Some(captured)
    }

    fn side_is_empty(&self, player: Player) -> bool {
        self.pits
            .iter()
            .filter(|pit| pit.owner == player && pit.is_house())
            .all(|pit| pit.stones == 0)
    }

    fn finish(&mut self) -> Option<Outcome> {
        if !(0..PLAYERS).any(|player| self.side_is_empty(player)) {
            return None;
        }
        let scores: Vec<i32> = (0..PLAYERS)
            .map(|player| {
                let remaining: u16 = self
                    .pits
                    .iter_mut()
                    .filter(|pit| pit.owner == player && pit.is_house())
                    .map(Pit::take_stones)
                    .sum();
                let store = self.store_mut(player);
                store.stones += remaining;
                i32::from(store.stones)
            })
            .collect();
        let best = scores.iter().max().expect("the board has players");
        Some(Outcome {
            winners: (0..PLAYERS)
                .filter(|&player| scores[player] == *best)
                .collect(),
            scores: Some(scores),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(as = MancalaAction)]
pub struct Action {
    pub pit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mancala {
    board: Board,
    turn: Turn,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = MancalaView)]
pub struct View {
    pub board: [u16; BOARD_SIZE],
}

impl Game for Mancala {
    type Config = crate::game::GameConfig;
    type Action = Action;
    type Event = Event;
    type View = View;
    fn with_config(players: usize, _: Seed, _: Self::Config) -> Result<Self, Error> {
        if players != PLAYERS {
            return Err(Error::PlayerCount);
        }
        Ok(Self {
            board: Board::new(),
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
        self.board
            .pits
            .iter()
            .enumerate()
            .filter(|(_, pit)| pit.owner == player && pit.is_house() && pit.stones > 0)
            .map(|(pit, _)| Action { pit })
            .collect()
    }
    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        let landing = self.board.sow(player, action.pit);
        let mut events = vec![Event::Played { player }];
        if let Some(count) = self.board.capture(player, landing) {
            events.push(Event::Captured {
                player,
                count: usize::from(count),
            });
        }
        self.outcome = self.board.finish();
        if self.board.pits[landing].is_store_for(player) {
            self.turn.number += 1;
        } else {
            self.turn.advance(PLAYERS);
        }
        events
    }
    fn view(&self, _: Player) -> View {
        View {
            board: self.board.stones(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    #[test]
    fn pit_layout_identifies_stores_and_opposites() {
        let board = Board::new();
        for (index, pit) in board.pits.iter().enumerate() {
            match pit.kind {
                PitKind::House { opposite } => {
                    let other = &board.pits[opposite];
                    assert_ne!(pit.owner, other.owner);
                    assert!(matches!(other.kind, PitKind::House { opposite } if opposite == index));
                    assert_eq!(pit.stones, INITIAL_STONES);
                }
                PitKind::Store => assert_eq!(pit.stones, 0),
            }
        }
        assert!(board.pits[6].is_store_for(0));
        assert!(board.pits[13].is_store_for(1));
    }

    #[test]
    fn sowing_wraps_and_skips_only_the_opponents_store() {
        for (player, start, own_store, opponent_store) in [(0, 5, 6, 13), (1, 12, 13, 6)] {
            let mut stones = [0; BOARD_SIZE];
            stones[start] = 14;
            let mut board = Board::from(stones);
            let landing = board.sow(player, start);
            assert_eq!(landing, own_store);
            assert_eq!(board.pits[own_store].stones, 2);
            assert_eq!(board.pits[opponent_store].stones, 0);
            assert!(
                board
                    .pits
                    .iter()
                    .filter(|pit| pit.is_house())
                    .all(|pit| pit.stones == 1)
            );
            assert_eq!(board.stones().iter().sum::<u16>(), 14);
        }
    }

    #[test]
    fn captures_work_for_both_players() {
        for (player, landing, opposite) in [(0, 1, 11), (1, 11, 1)] {
            let mut stones = [0; BOARD_SIZE];
            stones[landing] = 1;
            stones[opposite] = 3;
            let mut board = Board::from(stones);
            assert_eq!(board.capture(player, landing), Some(4));
            assert_eq!(board.pits[landing].stones, 0);
            assert_eq!(board.pits[opposite].stones, 0);
            assert_eq!(board.store_mut(player).stones, 4);
        }
    }

    #[test]
    fn capture_requires_own_previously_empty_house_and_nonempty_opposite() {
        for (player, landing_stones, opposite_stones) in [(1, 1, 3), (0, 2, 3), (0, 1, 0)] {
            let mut stones = [0; BOARD_SIZE];
            stones[1] = landing_stones;
            stones[11] = opposite_stones;
            let mut board = Board::from(stones);
            assert_eq!(board.capture(player, 1), None);
            assert_eq!(board.stones(), stones);
        }
        let mut board = Board::new();
        assert_eq!(board.capture(0, 6), None);
    }

    #[test]
    fn finishing_sweeps_either_side_and_allows_ties() {
        for empty_player in 0..PLAYERS {
            let mut board = Board::new();
            for pit in &mut board.pits {
                if pit.owner == empty_player && pit.is_house() {
                    pit.stones = 0;
                }
            }
            board.store_mut(empty_player).stones = 24;
            let outcome = board.finish().unwrap();
            assert_eq!(outcome.winners, vec![0, 1]);
            assert_eq!(outcome.scores, Some(vec![24, 24]));
            assert!(
                board
                    .pits
                    .iter()
                    .filter(|pit| pit.is_house())
                    .all(|pit| pit.stones == 0)
            );
        }
        assert!(Board::new().finish().is_none());
    }

    #[test]
    fn saved_state_keeps_the_original_flat_board_format() {
        let saved = serde_json::json!({
            "board": [1, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0],
            "turn": { "players": [0], "number": 0 },
            "outcome": null,
        });
        let mut game: Mancala = serde_json::from_value(saved.clone()).unwrap();
        assert_eq!(serde_json::to_value(&game).unwrap(), saved);
        assert_eq!(
            serde_json::to_value(game.view(0)).unwrap()["board"],
            saved["board"]
        );
        game.advance(0, &Action { pit: 0 });
        assert_eq!(game.outcome().unwrap().scores, Some(vec![4, 2]));
    }

    #[test]
    fn extra_turn_and_conservation() {
        let mut game = Session::<Mancala>::new(2, [0; 32]).unwrap();
        game.act(0, Action { pit: 2 }).unwrap();
        assert_eq!(game.state().turn().players, vec![0]);
        assert_eq!(game.state().board.stones().iter().sum::<u16>(), 48);
    }

    #[test]
    fn capture_and_end_sweep() {
        let mut game = Mancala::new(2, [0; 32]).unwrap();
        game.board = Board::from([1, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0]);
        let events = game.advance(0, &Action { pit: 0 });
        assert!(events.contains(&Event::Captured {
            player: 0,
            count: 4
        }));
        assert_eq!(game.board.store_mut(0).stones, 4);
        assert_eq!(game.board.store_mut(1).stones, 2);
        assert_eq!(game.outcome().unwrap().winners, vec![0]);
    }
}
