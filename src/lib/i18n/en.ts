/** English strings: the source of truth for every translation key. */
export const en = {
  'common.browse': 'Browse',
  'common.cancel': 'Cancel',
  'common.close': 'Close',
  'common.edit': 'Edit',
  'common.open': 'Open',
  'common.save': 'Save',
  'common.terminal': 'Terminal',
  'lang.en': 'English',
  'lang.es': 'Spanish',
  'lang.label': 'Language',
  'lang.system': 'System default',
} as const;

export type Key = keyof typeof en;
