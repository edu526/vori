import type { Theme } from '$lib/api/types';

function createThemeStore() {
  let theme = $state<Theme>('system');

  function applyScale(scale: number) {
    document.documentElement.style.zoom = String(scale);
  }

  function apply(t: Theme) {
    theme = t;
    const html = document.documentElement;
    if (t === 'system') {
      html.removeAttribute('data-theme');
      const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      html.classList.toggle('dark', isDark);
    } else {
      html.setAttribute('data-theme', t);
      html.classList.toggle('dark', t === 'dark');
    }
  }

  /** Clamps, applies, and returns the new scale value. Does NOT persist. */
  function step(delta: number, current: number): number {
    const rounded = Math.round(current * 10) / 10;
    const next = Math.max(0.8, Math.min(1.5, Math.round((rounded + delta) * 10) / 10));
    applyScale(next);
    return next;
  }

  return {
    get theme() {
      return theme;
    },
    apply,
    applyScale,
    step,
    init: async () => {},
  };
}

export const themeStore = createThemeStore();
