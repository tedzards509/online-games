import type { Schemas } from './types';

export const crazyEightsDefaults: Schemas['CrazyEightsGameConfig'] = {
  ace_are_you_strong: false,
  jack_skip: false,
  ten_reverse: false,
  two_draw_two: false,
};

/** Optional Crazy Eights rules; labels are `crazyEights.rules.<rule>`. */
export const crazyEightsRules = Object.keys(crazyEightsDefaults) as Array<
  keyof Schemas['CrazyEightsGameConfig']
>;
