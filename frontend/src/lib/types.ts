import type { components } from './api.generated';

export type Schemas = components['schemas'];
export type Snapshot = Schemas['Snapshot'];
export type GameSnapshot = Schemas['GameSnapshot'];
export type Membership = Schemas['Membership'];
export type Action = Schemas['Action'];
export type GameKind = Schemas['GameKind'];
export type Card = Schemas['Card'];
export type Suit = Schemas['Suit'];
export type ClientMessage = Schemas['ClientMessage'];
export type ServerMessage = Schemas['ServerMessage'];

export const games: Record<GameKind, string> = {
  tempel_des_schreckens: 'Tempel des Schreckens',
  the_crew: 'The Crew',
  mancala: 'Mancala',
  crazy_eights: 'Crazy Eights',
  shithead: 'Shithead',
  skyjo: 'Skyjo',
  tic_tac_toe: 'Tic-Tac-Toe',
};

export const suits: Record<Suit, string> = {
  clubs: '♣',
  diamonds: '♦',
  hearts: '♥',
  spades: '♠',
};

export function cardLabel(card: Card): string {
  const rank =
    ({ 11: 'J', 12: 'Q', 13: 'K', 14: 'A' } as Record<number, string>)[
      card.rank
    ] ?? card.rank;
  return `${rank}${suits[card.suit]}`;
}

/** Props every game board receives from GameBoard. Boards may require more. */
export type BoardProps<View> = {
  view: View;
  /** The viewer's legal actions; empty while a command is pending or offline. */
  actions: Action[];
  /** Nicknames by player index. */
  names: string[];
  /** The viewer's player index. */
  you: number;
  /** Player indices that may act now; empty after the game ends. */
  active: number[];
  finished: boolean;
  move: (action: Action) => void;
};
