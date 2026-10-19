use crate::game::{Error, Event, Game, Outcome, Player, Seed, Turn};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(as = MancalaAction)]
pub struct Action {
    pub pit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mancala {
    board: [u16; 14],
    turn: Turn,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = MancalaView)]
pub struct View {
    pub board: [u16; 14],
}

impl Game for Mancala {
    type Action = Action;
    type Event = Event;
    type View = View;
    fn new(players: usize, _: Seed) -> Result<Self, Error> {
        if players != 2 {
            return Err(Error::PlayerCount);
        }
        let mut board = [4; 14];
        board[6] = 0;
        board[13] = 0;
        Ok(Self {
            board,
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
        let start = player * 7;
        (start..start + 6)
            .filter(|&pit| self.board[pit] > 0)
            .map(|pit| Action { pit })
            .collect()
    }
    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        let store = player * 7 + 6;
        let mut stones = self.board[action.pit];
        self.board[action.pit] = 0;
        let mut pit = action.pit;
        while stones > 0 {
            pit = (pit + 1) % 14;
            if pit == (1 - player) * 7 + 6 {
                continue;
            }
            self.board[pit] += 1;
            stones -= 1;
        }
        let mut events = vec![Event::Played { player }];
        if (player * 7..player * 7 + 6).contains(&pit)
            && self.board[pit] == 1
            && self.board[12 - pit] > 0
        {
            let count = self.board[12 - pit] + 1;
            self.board[store] += count;
            self.board[pit] = 0;
            self.board[12 - pit] = 0;
            events.push(Event::Captured {
                player,
                count: count as usize,
            });
        }
        if self.board[..6].iter().all(|&n| n == 0) || self.board[7..13].iter().all(|&n| n == 0) {
            for p in 0..2 {
                for i in p * 7..p * 7 + 6 {
                    self.board[p * 7 + 6] += self.board[i];
                    self.board[i] = 0;
                }
            }
            let scores = vec![self.board[6] as i32, self.board[13] as i32];
            let best = *scores.iter().max().unwrap();
            self.outcome = Some(Outcome {
                winners: (0..2).filter(|&p| scores[p] == best).collect(),
                scores: Some(scores),
            });
        }
        if pit == store {
            self.turn.number += 1;
        } else {
            self.turn.advance(2);
        }
        events
    }
    fn view(&self, _: Player) -> View {
        View { board: self.board }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;
    #[test]
    fn extra_turn_and_conservation() {
        let mut game = Session::<Mancala>::new(2, [0; 32]).unwrap();
        game.act(0, Action { pit: 2 }).unwrap();
        assert_eq!(game.state().turn().players, vec![0]);
        assert_eq!(game.state().board.iter().sum::<u16>(), 48);
    }
    #[test]
    fn capture_and_end_sweep() {
        let mut game = Mancala::new(2, [0; 32]).unwrap();
        game.board = [0; 14];
        game.board[0] = 1;
        game.board[11] = 3;
        game.board[7] = 2;
        let events = game.advance(0, &Action { pit: 0 });
        assert!(events.contains(&Event::Captured {
            player: 0,
            count: 4
        }));
        assert_eq!(game.board[6], 4);
        assert_eq!(game.board[13], 2);
        assert_eq!(game.outcome().unwrap().winners, vec![0]);
    }
}
