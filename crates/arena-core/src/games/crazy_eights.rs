use crate::{
    cards::{Card, Deck, Suit, standard_deck},
    game::{Error, Event, Game, Outcome, Player, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const WILD_RANK: u8 = 8;

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
        game.active_suit = Suit::Hearts;
        assert_eq!(game.legal_actions(game.turn.players[0]).len(), 4);
        game.advance(
            game.turn.players[0],
            &Action::Play {
                index: 0,
                suit: Some(Suit::Diamonds),
            },
        );
        assert_eq!(game.active_suit, Suit::Diamonds);
        game.hands[1] = vec![card(2, Suit::Diamonds), card(11, Suit::Spades)];
        assert_eq!(
            game.legal_actions(game.turn.players[0]),
            vec![Action::Play {
                index: 0,
                suit: None
            }]
        );
        game.advance(
            game.turn.players[0],
            &Action::Play {
                index: 0,
                suit: None,
            },
        );
        assert_eq!(game.turn.players, vec![0]);
    }

    #[test]
    fn exhausted_stock_passes_until_everyone_is_blocked() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        game.deck = Deck::new(vec![]);
        game.discard = vec![card(3, Suit::Hearts)];
        game.active_suit = Suit::Hearts;
        game.hands = vec![vec![card(4, Suit::Clubs)], vec![card(5, Suit::Spades)]];
        assert_eq!(game.legal_actions(game.turn.players[0]), vec![Action::Draw]);
        game.advance(game.turn.players[0], &Action::Draw);
        assert!(game.outcome.is_none());
        game.advance(game.turn.players[0], &Action::Draw);
        assert!(game.outcome.unwrap().winners.is_empty());
    }

    #[test]
    fn drawing_recycles_below_the_top_and_ends_the_turn() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        let recycled = card(4, Suit::Hearts);
        let top = card(3, Suit::Hearts);
        game.deck = Deck::new(vec![]);
        game.discard = vec![recycled, top];
        game.active_suit = Suit::Hearts;
        game.hands[0] = vec![card(5, Suit::Clubs)];
        game.consecutive_passes = 1;

        assert_eq!(game.legal_actions(0), vec![Action::Draw]);
        game.advance(0, &Action::Draw);

        assert_eq!(game.discard, vec![top]);
        assert_eq!(game.hands[0], vec![card(5, Suit::Clubs), recycled]);
        assert_eq!(game.consecutive_passes, 0);
        assert_eq!(game.turn.players, vec![1]);
        assert!(game.outcome.is_none());
    }

    #[test]
    fn playing_the_last_card_wins_and_resets_passes() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        game.hands[0] = vec![card(WILD_RANK, Suit::Clubs)];
        game.consecutive_passes = 1;
        game.advance(
            0,
            &Action::Play {
                index: 0,
                suit: Some(Suit::Spades),
            },
        );
        assert_eq!(game.active_suit, Suit::Spades);
        assert_eq!(game.consecutive_passes, 0);
        assert_eq!(game.outcome.as_ref().unwrap().winners, vec![0]);
        assert!(game.legal_actions(1).is_empty());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrazyEights {
    hands: Vec<Vec<Card>>,
    deck: Deck<Card>,
    discard: Vec<Card>,
    #[serde(rename = "suit")]
    active_suit: Suit,
    rng: ChaCha12Rng,
    turn: Turn,
    #[serde(rename = "blocked")]
    consecutive_passes: usize,
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

impl CrazyEights {
    fn top_card(&self) -> Card {
        *self
            .discard
            .last()
            .expect("the discard always has a top card")
    }

    fn play_card(&mut self, player: Player, index: usize, chosen_suit: Option<Suit>) {
        let card = self.hands[player].remove(index);
        self.active_suit = chosen_suit.unwrap_or(card.suit);
        self.discard.push(card);
        self.consecutive_passes = 0;
        if self.hands[player].is_empty() {
            self.outcome = Some(Outcome {
                winners: vec![player],
                scores: None,
            });
        }
    }

    fn draw_or_pass(&mut self, player: Player) {
        self.deck.recycle(&mut self.discard, &mut self.rng);
        if let Some(card) = self.deck.draw() {
            self.hands[player].push(card);
            self.consecutive_passes = 0;
        } else {
            self.consecutive_passes += 1;
        }
        if self.consecutive_passes == self.hands.len() {
            self.outcome = Some(Outcome {
                winners: vec![],
                scores: None,
            });
        }
    }
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
        let cards_per_player = if players == 2 { 7 } else { 5 };
        let hands = (0..players)
            .map(|_| {
                (0..cards_per_player)
                    .map(|_| deck.draw().expect("the deck can deal every hand"))
                    .collect()
            })
            .collect();
        let top = deck.draw().expect("the deal leaves an opening discard");
        Ok(Self {
            hands,
            deck,
            discard: vec![top],
            active_suit: top.suit,
            rng,
            turn: Turn::first(),
            consecutive_passes: 0,
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
        let top = self.top_card();
        let mut actions = vec![];
        for (index, card) in self.hands[player].iter().enumerate() {
            if card.rank == WILD_RANK {
                actions.extend(Suit::ALL.map(|suit| Action::Play {
                    index,
                    suit: Some(suit),
                }));
            } else if card.suit == self.active_suit || card.rank == top.rank {
                actions.push(Action::Play { index, suit: None });
            }
        }
        if actions.is_empty() {
            vec![Action::Draw]
        } else {
            actions
        }
    }
    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        match *action {
            Action::Play { index, suit } => self.play_card(player, index, suit),
            Action::Draw => self.draw_or_pass(player),
        }
        self.turn.advance(self.hands.len());
        vec![Event::Played { player }]
    }
    fn view(&self, viewer: Player) -> View {
        View {
            hand: self.hands.get(viewer).cloned().unwrap_or_default(),
            counts: self.hands.iter().map(Vec::len).collect(),
            top: self.top_card(),
            suit: self.active_suit,
            stock: self.deck.len(),
        }
    }
}
