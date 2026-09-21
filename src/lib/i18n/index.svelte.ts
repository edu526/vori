import type { Language } from '$lib/api/types';
import { en, type Key } from './en';
import { es } from './es';

export type Locale = 'en' | 'es';
export type { Key };

function systemLocale(): Locale {
  try {
    return navigator.language.toLowerCase().startsWith('es') ? 'es' : 'en';
  } catch {
    return 'en';
  }
}

let preference = $state<Language>('system');
const locale = $derived<Locale>(preference === 'system' ? systemLocale() : preference);

type Params = Record<string, string | number>;

function fill(text: string, params?: Params): string {
  return params ? text.replace(/\{(\w+)\}/g, (_, name) => String(params[name] ?? `{${name}}`)) : text;
}

/** Translate `key`, replacing `{name}` placeholders. Falls back to English. */
export function t(key: Key, params?: Params): string {
  return fill((locale === 'es' ? es : en)[key] ?? en[key], params);
}

/** `one` for a count of 1, `other` otherwise; `{count}` is filled in automatically. */
export function tn(one: Key, other: Key, count: number, params: Params = {}): string {
  return t(count === 1 ? one : other, { count, ...params });
}

export const i18n = {
  get locale(): Locale {
    return locale;
  },
  get preference(): Language {
    return preference;
  },
  setPreference(next: Language) {
    preference = next;
    try {
      document.documentElement.lang = locale;
    } catch {
      /* no document (tests) */
    }
  },
};
