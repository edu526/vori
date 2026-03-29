import type { AppData } from '$lib/api/types';

export const mockAppData: AppData = {
  categories: {
    personal: { parent: null },
    'personal/web': { parent: 'personal' },
    'personal/tools': { parent: 'personal' },
    work: { parent: null },
    'work/frontend': { parent: 'work' },
    'work/backend': { parent: 'work' },
    open_source: { parent: null },
  },
  projects: {
    vori: {
      path: '/home/user/projects/vori',
      parent: 'personal/tools',
    },
    portfolio: {
      path: '/home/user/projects/portfolio',
      parent: 'personal/web',
    },
    dotfiles: {
      path: '/home/user/dotfiles',
      parent: 'personal',
    },
    dashboard: {
      path: '/home/user/work/dashboard',
      parent: 'work/frontend',
    },
    'api-gateway': {
      path: '/home/user/work/api-gateway',
      parent: 'work/backend',
    },
    'auth-service': {
      path: '/home/user/work/auth-service',
      parent: 'work/backend',
    },
    'svelte-utils': {
      path: '/home/user/oss/svelte-utils',
      parent: 'open_source',
    },
  },
  files: {
    zshrc: { path: '/home/user/.zshrc' },
    vimrc: { path: '/home/user/.vimrc' },
    hosts: { path: '/etc/hosts' },
  },
  preferences: {
    default_editor: 'vscode',
    default_text_editor: 'vscode',
    close_on_open_editor: false,
    close_on_open_terminal: false,
    close_on_open_file: false,
    terminal: {
      preferred: 'kitty',
      available: { kitty: '/usr/bin/kitty', gnome_terminal: '/usr/bin/gnome-terminal' },
      last_detected: 'kitty',
    },
    editors_available: {
      vscode: '/usr/bin/code',
      cursor: '/usr/bin/cursor',
    },
    theme: 'system',
    autostart: true,
    show_tray: true,
    keep_background: true,
    hotkey: 'Super+Shift+KeyV',
    ui_scale: 1.0,
  },
  favorites: {
    projects: ['vori', 'dashboard'],
    files: ['zshrc'],
    categories: [],
  },
  recents: [
    { path: '/home/user/work/dashboard', name: 'dashboard', type: 'project', timestamp: Date.now() / 1000 - 3600 },
    { path: '/home/user/projects/vori', name: 'vori', type: 'project', timestamp: Date.now() / 1000 - 7200 },
    { path: '/home/user/.zshrc', name: 'zshrc', type: 'file', timestamp: Date.now() / 1000 - 86400 },
  ],
};
