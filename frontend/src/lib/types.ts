import type { components } from './api.generated';

export type Schemas = components['schemas'];
export type Snapshot = Schemas['Snapshot'];
export type Membership = Schemas['Membership'];
export type Action = Schemas['Action'];
export type GameKind = Schemas['GameKind'];
export type Card = Schemas['Card'];
export type Suit = Schemas['Suit'];
export type ClientMessage = Schemas['ClientMessage'];
export type ServerMessage = Schemas['ServerMessage'];

export const games: Record<GameKind, string> = {
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
