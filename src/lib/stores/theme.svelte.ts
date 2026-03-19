import type { Theme } from '$lib/api/types';

function createThemeStore() {
  let theme = $state<Theme>('system');

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

  return {
    get theme() {
      return theme;
    },
    apply,
    init: async () => {},
  };
}

export const themeStore = createThemeStore();
