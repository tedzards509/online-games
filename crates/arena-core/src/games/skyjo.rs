use crate::{
    cards::Deck,
    game::{Error, Event, Game, Outcome, Player, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Stock,
    Discard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
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
pub struct Skyjo {
    grids: Vec<Vec<Option<Cell>>>,
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
pub struct View {
    pub grids: Vec<Vec<CellView>>,
    pub top: Option<i8>,
    pub stock: usize,
    pub pending: Option<i8>,
    pub phase: Phase,
    pub finisher: Option<Player>,
}

impl Skyjo {
    fn eliminate(&mut self, player: Player, events: &mut Vec<Event>) {
        for column in 0..4 {
            let indices = [column, column + 4, column + 8];
            if let Some(first) = self.grids[player][column] {
                if first.revealed
                    && indices.iter().all(|&i| {
                        self.grids[player][i].is_some_and(|c| c.revealed && c.value == first.value)
                    })
                {
                    for i in indices {
                        self.discard
                            .push(self.grids[player][i].take().unwrap().value);
                    }
                    events.push(Event::ColumnRemoved { player, column });
                }
            }
        }
    }
    fn end_turn(&mut self, events: &mut Vec<Event>) {
        self.eliminate(self.turn.player, events);
        if self.finisher.is_none()
            && self.grids[self.turn.player]
                .iter()
                .all(|c| c.is_none_or(|c| c.revealed))
        {
            self.finisher = Some(self.turn.player);
        }
        self.turn.next(self.grids.len());
        self.phase = Phase::Draw;
        if self.finisher == Some(self.turn.player) {
            for player in 0..self.grids.len() {
                for cell in self.grids[player].iter_mut().flatten() {
                    cell.revealed = true;
                }
                self.eliminate(player, events);
            }
            let mut scores: Vec<i32> = self
                .grids
                .iter()
                .map(|g| g.iter().flatten().map(|c| c.value as i32).sum())
                .collect();
            let finisher = self.finisher.unwrap();
            if scores[finisher] > 0
                && scores
                    .iter()
                    .enumerate()
                    .any(|(p, &score)| p != finisher && score <= scores[finisher])
            {
                scores[finisher] *= 2;
            }
            let best = *scores.iter().min().unwrap();
            self.outcome = Some(Outcome {
                winners: (0..scores.len()).filter(|&p| scores[p] == best).collect(),
                scores: Some(scores),
            });
        }
    }
}

impl Game for Skyjo {
    type Action = Action;
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
        let grids = (0..players)
            .map(|_| {
                (0..12)
                    .map(|_| {
                        Some(Cell {
                            value: deck.draw().unwrap(),
                            revealed: false,
                        })
                    })
                    .collect()
            })
            .collect();
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
        self.turn
    }
    fn outcome(&self) -> Option<Outcome> {
        self.outcome.clone()
    }
    fn legal_actions(&self) -> Vec<Action> {
        if self.outcome.is_some() {
            return vec![];
        }
        let grid = &self.grids[self.turn.player];
        match self.phase {
            Phase::Setup | Phase::Reveal => grid
                .iter()
                .enumerate()
                .filter(|(_, c)| c.is_some_and(|c| !c.revealed))
                .map(|(index, _)| Action::Reveal { index })
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
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_some())
                    .map(|(index, _)| Action::Replace { index })
                    .collect();
                if self.source == Source::Stock && grid.iter().flatten().any(|c| !c.revealed) {
                    actions.push(Action::Discard);
                }
                actions
            }
        }
    }
    fn advance(&mut self, action: &Action) -> Vec<Event> {
        let player = self.turn.player;
        let mut events = vec![Event::Played { player }];
        match *action {
            Action::Reveal { index } => {
                self.grids[player][index].as_mut().unwrap().revealed = true;
                if self.phase == Phase::Setup {
                    if self.grids[player]
                        .iter()
                        .flatten()
                        .filter(|c| c.revealed)
                        .count()
                        == 2
                    {
                        self.turn.next(self.grids.len());
                        if self.turn.player == 0 {
                            self.phase = Phase::Draw;
                            self.turn.player = (0..self.grids.len())
                                .max_by_key(|&p| {
                                    (
                                        self.grids[p]
                                            .iter()
                                            .flatten()
                                            .filter(|c| c.revealed)
                                            .map(|c| c.value as i32)
                                            .sum::<i32>(),
                                        std::cmp::Reverse(p),
                                    )
                                })
                                .unwrap();
                        }
                    }
                } else {
                    self.end_turn(&mut events);
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
                let cell = self.grids[player][index].as_mut().unwrap();
                self.discard.push(cell.value);
                cell.value = self.pending.take().unwrap();
                cell.revealed = true;
                self.end_turn(&mut events);
            }
            Action::Discard => {
                self.discard.push(self.pending.take().unwrap());
                self.phase = Phase::Reveal;
            }
        }
        events
    }
    fn view(&self, viewer: Player) -> View {
        View {
            grids: self
                .grids
                .iter()
                .map(|grid| {
                    grid.iter()
                        .map(|c| CellView {
                            value: c.filter(|c| c.revealed).map(|c| c.value),
                            removed: c.is_none(),
                        })
                        .collect()
                })
                .collect(),
            top: self.discard.last().copied(),
            stock: self.deck.len(),
            pending: if viewer == self.turn.player {
                self.pending
            } else {
                None
            },
            phase: self.phase,
            finisher: self.finisher,
        }
    }
}
