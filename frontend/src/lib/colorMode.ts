import { get, writable } from 'svelte/store';

/** Device follows `prefers-color-scheme`; light and dark are explicit choices. */
export const colorModes = ['device', 'light', 'dark'] as const;
export type ColorMode = (typeof colorModes)[number];
const storageKey = 'arena-color-mode';
const darkDevice = matchMedia('(prefers-color-scheme: dark)');

function isColorMode(value: string | null): value is ColorMode {
  return colorModes.some((mode) => mode === value);
}

function savedColorMode(): ColorMode {
  try {
    const saved = localStorage.getItem(storageKey);
    if (isColorMode(saved)) return saved;
  } catch {
    // The device mode still works when browser storage is unavailable.
  }
  return 'device';
}

export const colorMode = writable<ColorMode>(savedColorMode());

/** Sets `<html data-mode>` to the resolved `light` or `dark` theme mode. */
function apply(mode: ColorMode): void {
  const dark = mode === 'dark' || (mode === 'device' && darkDevice.matches);
  document.documentElement.dataset.mode = dark ? 'dark' : 'light';
}
colorMode.subscribe(apply);
darkDevice.addEventListener('change', () => apply(get(colorMode)));

/** Apply immediately and remember the selection on this device. */
export function setColorMode(mode: ColorMode): void {
  colorMode.set(mode);
  try {
    localStorage.setItem(storageKey, mode);
  } catch {
    // The selection remains usable for the current visit.
  }
}
