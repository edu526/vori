import type { AppData } from '$lib/api/types';

export const mockAppData: AppData = {
  categories: {
    personal: {
      description: 'Personal projects',
      icon: '🏠',
      subcategories: {
        'personal/web': { description: 'Web experiments', icon: '🌐' },
        'personal/tools': { description: 'CLI tools & scripts', icon: '🔧' },
      },
    },
    work: {
      description: 'Work projects',
      icon: '💼',
      subcategories: {
        'work/frontend': { description: 'Frontend apps', icon: '🎨' },
        'work/backend': { description: 'Backend services', icon: '⚙️' },
      },
    },
    open_source: {
      description: 'Open source contributions',
      icon: '🌍',
      subcategories: {},
    },
  },
  projects: {
    'vori': {
      path: '/home/user/projects/vori',
      category: 'personal',
      subcategory: 'personal/tools',
    },
    'portfolio': {
      path: '/home/user/projects/portfolio',
      category: 'personal',
      subcategory: 'personal/web',
    },
    'dotfiles': {
      path: '/home/user/dotfiles',
      category: 'personal',
    },
    'dashboard': {
      path: '/home/user/work/dashboard',
      category: 'work',
      subcategory: 'work/frontend',
    },
    'api-gateway': {
      path: '/home/user/work/api-gateway',
      category: 'work',
      subcategory: 'work/backend',
    },
    'auth-service': {
      path: '/home/user/work/auth-service',
      category: 'work',
      subcategory: 'work/backend',
    },
    'svelte-utils': {
      path: '/home/user/oss/svelte-utils',
      category: 'open_source',
    },
  },
  files: {
    'zshrc': { path: '/home/user/.zshrc' },
    'vimrc': { path: '/home/user/.vimrc' },
    'hosts': { path: '/etc/hosts' },
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
