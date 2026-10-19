use crate::{
    cards::Deck,
    game::{Error, Event, Game, Outcome, Player, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const GRID_ROWS: usize = 3;
const GRID_COLUMNS: usize = 4;
const GRID_SIZE: usize = GRID_ROWS * GRID_COLUMNS;
const SETUP_REVEALS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Stock,
    Discard,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(value: i8, revealed: bool) -> Option<Cell> {
        Some(Cell { value, revealed })
    }

    #[test]
    fn distribution_has_150_cards() {
        let game = Skyjo::new(8, [0; 32]).unwrap();
        let mut values = vec![];
        let mut deck = game.deck.clone();
        while let Some(value) = deck.draw() {
            values.push(value);
        }
        values.extend(game.discard);
        values.extend(
            game.grids
                .into_iter()
                .flat_map(|grid| grid.cells)
                .flatten()
                .map(|cell| cell.value),
        );
        assert_eq!(values.len(), 150);
        assert_eq!(values.iter().filter(|&&v| v == -2).count(), 5);
        assert_eq!(values.iter().filter(|&&v| v == 0).count(), 15);
        for value in -1..=12 {
            if value != 0 {
                assert_eq!(values.iter().filter(|&&v| v == value).count(), 10);
            }
        }
    }

    #[test]
    fn stock_card_is_private_and_discard_must_be_replaced() {
        let mut game = Skyjo::new(2, [0; 32]).unwrap();
        game.phase = Phase::Draw;
        game.advance(
            0,
            &Action::Draw {
                source: Source::Stock,
            },
        );
        assert!(game.view(0).pending.is_some());
        assert!(game.view(1).pending.is_none());
        assert!(game.legal_actions(0).contains(&Action::Discard));
        game.advance(0, &Action::Discard);
        assert_eq!(game.phase, Phase::Reveal);
        assert!(
            game.legal_actions(0)
                .iter()
                .all(|a| matches!(a, Action::Reveal { .. }))
        );
        game.advance(0, &Action::Reveal { index: 0 });
        game.advance(
            1,
            &Action::Draw {
                source: Source::Discard,
            },
        );
        assert!(!game.legal_actions(1).contains(&Action::Discard));
    }

    #[test]
    fn matching_columns_are_removed_and_final_turn_scores_double_on_tie() {
        let mut game = Skyjo::new(2, [0; 32]).unwrap();
        game.grids = vec![
            Grid {
                cells: vec![None; GRID_SIZE]
            };
            2
        ];
        for i in [0, 4, 8] {
            game.grids[0].cells[i] = cell(7, true);
        }
        game.grids[0].cells[1] = cell(5, true);
        game.grids[1].cells[1] = cell(5, false);
        let mut events = vec![];
        game.end_turn(0, &mut events);
        assert!(events.contains(&Event::ColumnRemoved {
            player: 0,
            column: 0
        }));
        assert_eq!(game.finisher, Some(0));
        assert!(game.outcome.is_none());
        game.grids[1].reveal(1);
        game.end_turn(1, &mut events);
        assert_eq!(
            game.outcome.unwrap(),
            Outcome {
                winners: vec![1],
                scores: Some(vec![10, 5])
            }
        );
    }

    #[test]
    fn columns_require_three_equal_revealed_cards() {
        let mut grid = Grid {
            cells: vec![None; GRID_SIZE],
        };
        for index in [2, 6, 10] {
            grid.cells[index] = cell(7, true);
        }
        grid.cells[10] = cell(7, false);
        assert!(grid.take_matching_column(2).is_none());
        grid.reveal(10);
        grid.cells[6] = cell(8, true);
        assert!(grid.take_matching_column(2).is_none());
        grid.cells[6] = cell(7, true);
        assert_eq!(grid.take_matching_column(2), Some([7; GRID_ROWS]));
        assert!(grid.take_matching_column(2).is_none());
        assert!(grid.fully_revealed());
        assert_eq!(grid.score(), 0);
        assert!(
            grid.view()
                .iter()
                .all(|cell| cell.removed && cell.value.is_none())
        );
    }

    #[test]
    fn setup_uses_two_reveals_then_highest_sum_with_earliest_seat_breaking_ties() {
        for (values, expected_starter) in [([3, 7, 5, 5, 1, 2], 0), ([1, 2, 5, 5, 3, 4], 1)] {
            let mut game = Skyjo::new(3, [0; 32]).unwrap();
            for (player, grid) in game.grids.iter_mut().enumerate() {
                grid.cells[0] = cell(values[player * SETUP_REVEALS], false);
                grid.cells[1] = cell(values[player * SETUP_REVEALS + 1], false);
            }
            for player in 0..3 {
                game.advance(player, &Action::Reveal { index: 0 });
                assert_eq!(game.phase, Phase::Setup);
                assert_eq!(game.turn.players, vec![player]);
                game.advance(player, &Action::Reveal { index: 1 });
            }
            assert_eq!(game.phase, Phase::Draw);
            assert_eq!(game.turn.players, vec![expected_starter]);
            assert_eq!(game.turn.number, 3);
        }
    }

    #[test]
    fn finisher_penalty_requires_a_positive_score_and_an_opponent_scoring_as_low() {
        for (finisher, opponent, expected) in
            [(3, 4, 3), (3, 3, 6), (3, 2, 6), (0, -1, 0), (-1, -2, -1)]
        {
            let mut game = Skyjo::new(2, [0; 32]).unwrap();
            for (grid, value) in game.grids.iter_mut().zip([finisher, opponent]) {
                grid.cells = vec![None; GRID_SIZE];
                grid.cells[0] = cell(value, false);
            }
            game.finish_round(0, &mut vec![]);
            assert_eq!(
                game.outcome.unwrap().scores,
                Some(vec![expected, i32::from(opponent)])
            );
            assert!(game.grids.iter().all(Grid::fully_revealed));
        }
    }

    #[test]
    fn exhausted_stock_is_unavailable_but_recyclable_discards_enable_a_draw() {
        let mut game = Skyjo::new(2, [0; 32]).unwrap();
        game.phase = Phase::Draw;
        game.deck = Deck::new(vec![]);
        game.discard = vec![5];
        assert_eq!(
            game.legal_actions(0),
            vec![Action::Draw {
                source: Source::Discard
            }]
        );
        game.discard.insert(0, 7);
        assert_eq!(
            game.legal_actions(0),
            vec![
                Action::Draw {
                    source: Source::Discard
                },
                Action::Draw {
                    source: Source::Stock
                }
            ]
        );
        game.advance(
            0,
            &Action::Draw {
                source: Source::Stock,
            },
        );
        assert_eq!(game.pending, Some(7));
        assert_eq!(game.discard, vec![5]);
        assert_eq!(game.phase, Phase::Place);
        assert_eq!(game.turn.players, vec![0]);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schema(as = SkyjoAction)]
pub enum Action {
    Reveal { index: usize },
    Draw { source: Source },
    Replace { index: usize },
    Discard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Setup,
    Draw,
    Place,
    Reveal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Cell {
    value: i8,
    revealed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct Grid {
    cells: Vec<Option<Cell>>,
}

impl Grid {
    fn deal(deck: &mut Deck<i8>) -> Self {
        Self {
            cells: (0..GRID_SIZE)
                .map(|_| {
                    Some(Cell {
                        value: deck.draw().expect("the deck can deal every grid"),
                        revealed: false,
                    })
                })
                .collect(),
        }
    }

    fn hidden_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.is_some_and(|cell| !cell.revealed))
            .map(|(index, _)| index)
    }

    fn revealed_count(&self) -> usize {
        self.cells
            .iter()
            .flatten()
            .filter(|cell| cell.revealed)
            .count()
    }

    fn revealed_score(&self) -> i32 {
        self.cells
            .iter()
            .flatten()
            .filter(|cell| cell.revealed)
            .map(|cell| i32::from(cell.value))
            .sum()
    }

    fn score(&self) -> i32 {
        self.cells
            .iter()
            .flatten()
            .map(|cell| i32::from(cell.value))
            .sum()
    }

    fn fully_revealed(&self) -> bool {
        self.hidden_indices().next().is_none()
    }

    fn reveal(&mut self, index: usize) {
        self.cells[index]
            .as_mut()
            .expect("only occupied cells can be revealed")
            .revealed = true;
    }

    fn reveal_all(&mut self) {
        for cell in self.cells.iter_mut().flatten() {
            cell.revealed = true;
        }
    }

    fn replace(&mut self, index: usize, value: i8) -> i8 {
        let cell = self.cells[index]
            .as_mut()
            .expect("only occupied cells can be replaced");
        let previous = cell.value;
        *cell = Cell {
            value,
            revealed: true,
        };
        previous
    }

    fn take_matching_column(&mut self, column: usize) -> Option<[i8; GRID_ROWS]> {
        let indices: [usize; GRID_ROWS] = std::array::from_fn(|row| row * GRID_COLUMNS + column);
        let first = self.cells[indices[0]]?;
        let matches = indices.iter().all(|&index| {
            self.cells[index].is_some_and(|cell| cell.revealed && cell.value == first.value)
        });
        if !matches {
            return None;
        }
        Some(indices.map(|index| {
            self.cells[index]
                .take()
                .expect("matching columns are occupied")
                .value
        }))
    }

    fn view(&self) -> Vec<CellView> {
        self.cells
            .iter()
            .map(|cell| CellView {
                value: cell.filter(|cell| cell.revealed).map(|cell| cell.value),
                removed: cell.is_none(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skyjo {
    grids: Vec<Grid>,
    deck: Deck<i8>,
    discard: Vec<i8>,
    rng: ChaCha12Rng,
    turn: Turn,
    phase: Phase,
    pending: Option<i8>,
    source: Source,
    finisher: Option<Player>,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CellView {
    pub value: Option<i8>,
    pub removed: bool,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = SkyjoView)]
pub struct View {
    pub grids: Vec<Vec<CellView>>,
    pub top: Option<i8>,
    pub stock: usize,
    pub pending: Option<i8>,
    pub phase: Phase,
    pub finisher: Option<Player>,
}

impl Skyjo {
    fn remove_matching_columns(&mut self, player: Player, events: &mut Vec<Event>) {
        for column in 0..GRID_COLUMNS {
            if let Some(values) = self.grids[player].take_matching_column(column) {
                self.discard.extend(values);
                events.push(Event::ColumnRemoved { player, column });
            }
        }
    }
    fn end_turn(&mut self, player: Player, events: &mut Vec<Event>) {
        self.remove_matching_columns(player, events);
        if self.finisher.is_none() && self.grids[player].fully_revealed() {
            self.finisher = Some(player);
        }
        self.turn.advance(self.grids.len());
        self.phase = Phase::Draw;
        if let Some(finisher) = self.finisher
            && self.turn.players[0] == finisher
        {
            self.finish_round(finisher, events);
        }
    }

    fn finish_round(&mut self, finisher: Player, events: &mut Vec<Event>) {
        for player in 0..self.grids.len() {
            self.grids[player].reveal_all();
            self.remove_matching_columns(player, events);
        }
        let mut scores: Vec<i32> = self.grids.iter().map(Grid::score).collect();
        let finisher_score = scores[finisher];
        let opponent_scored_as_low = scores
            .iter()
            .enumerate()
            .any(|(player, &score)| player != finisher && score <= finisher_score);
        if finisher_score > 0 && opponent_scored_as_low {
            scores[finisher] *= 2;
        }
        let best = *scores.iter().min().expect("the round has players");
        self.outcome = Some(Outcome {
            winners: (0..scores.len())
                .filter(|&player| scores[player] == best)
                .collect(),
            scores: Some(scores),
        });
    }

    fn finish_setup_turn(&mut self, player: Player) {
        if self.grids[player].revealed_count() != SETUP_REVEALS {
            return;
        }
        self.turn.advance(self.grids.len());
        if self.turn.players[0] == 0 {
            self.phase = Phase::Draw;
            // Highest revealed sum starts; the earlier seat wins a tie.
            self.turn.players[0] = self
                .grids
                .iter()
                .enumerate()
                .max_by_key(|(player, grid)| (grid.revealed_score(), std::cmp::Reverse(*player)))
                .map(|(player, _)| player)
                .expect("the game has players");
        }
    }
}

impl Game for Skyjo {
    type Action = Action;
    type Event = Event;
    type View = View;
    fn new(players: usize, seed: Seed) -> Result<Self, Error> {
        if !(2..=8).contains(&players) {
            return Err(Error::PlayerCount);
        }
        let mut rng = ChaCha12Rng::from_seed(seed);
        let mut deck = Deck::new(
            (-2..=12)
                .flat_map(|value| {
                    std::iter::repeat_n(
                        value,
                        match value {
                            -2 => 5,
                            0 => 15,
                            _ => 10,
                        },
                    )
                })
                .collect(),
        );
        deck.shuffle(&mut rng);
        let grids = (0..players).map(|_| Grid::deal(&mut deck)).collect();
        let discard = vec![deck.draw().unwrap()];
        Ok(Self {
            grids,
            deck,
            discard,
            rng,
            turn: Turn::first(),
            phase: Phase::Setup,
            pending: None,
            source: Source::Stock,
            finisher: None,
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
        let grid = &self.grids[player];
        match self.phase {
            Phase::Setup | Phase::Reveal => grid
                .hidden_indices()
                .map(|index| Action::Reveal { index })
                .collect(),
            Phase::Draw => {
                let mut actions = vec![Action::Draw {
                    source: Source::Discard,
                }];
                if !self.deck.is_empty() || self.discard.len() > 1 {
                    actions.push(Action::Draw {
                        source: Source::Stock,
                    });
                }
                actions
            }
            Phase::Place => {
                let mut actions: Vec<_> = grid
                    .cells
                    .iter()
                    .enumerate()
                    .filter(|(_, cell)| cell.is_some())
                    .map(|(index, _)| Action::Replace { index })
                    .collect();
                if self.source == Source::Stock && !grid.fully_revealed() {
                    actions.push(Action::Discard);
                }
                actions
            }
        }
    }
    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        let mut events = vec![Event::Played { player }];
        match *action {
            Action::Reveal { index } => {
                self.grids[player].reveal(index);
                if self.phase == Phase::Setup {
                    self.finish_setup_turn(player);
                } else {
                    self.end_turn(player, &mut events);
                }
            }
            Action::Draw { source } => {
                self.source = source;
                self.pending = match source {
                    Source::Discard => self.discard.pop(),
                    Source::Stock => {
                        self.deck.recycle(&mut self.discard, &mut self.rng);
                        self.deck.draw()
                    }
                };
                self.phase = Phase::Place;
            }
            Action::Replace { index } => {
                let value = self.pending.take().expect("replacement follows a draw");
                let previous = self.grids[player].replace(index, value);
                self.discard.push(previous);
                self.end_turn(player, &mut events);
            }
            Action::Discard => {
                self.discard.push(
                    self.pending
                        .take()
                        .expect("discarding follows a stock draw"),
                );
                self.phase = Phase::Reveal;
            }
        }
        events
    }
    fn view(&self, viewer: Player) -> View {
        View {
            grids: self.grids.iter().map(Grid::view).collect(),
            top: self.discard.last().copied(),
            stock: self.deck.len(),
            pending: if self.turn.players.contains(&viewer) {
                self.pending
            } else {
                None
            },
            phase: self.phase,
            finisher: self.finisher,
        }
    }
}
