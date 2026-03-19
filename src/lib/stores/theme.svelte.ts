import type { Theme } from '$lib/api/types';

function createThemeStore() {
  let theme = $state<Theme>('system');

  function apply(t: Theme) {
    theme = t;
    if (t === 'system') {
      document.documentElement.removeAttribute('data-theme');
    } else {
      document.documentElement.setAttribute('data-theme', t);
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
