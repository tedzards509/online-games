import type { Schemas } from './types';

export const crazyEightsDefaults: Schemas['CrazyEightsGameConfig'] = {
  ace_are_you_strong: false,
  jack_skip: false,
  ten_reverse: false,
  eight_wild: true,
  two_draw_two: false,
};

export const crazyEightsRules: Record<
  keyof Schemas['CrazyEightsGameConfig'],
  string
> = {
  ace_are_you_strong: 'A: Are you strong — play an ace or draw two',
  jack_skip: 'J: Skip next player',
  ten_reverse: '10: Reverse (skip with two players)',
  eight_wild: '8: Choose suit',
  two_draw_two: '2: Next player draws two',
};
