import type { Locale } from './index.svelte';

/** Spanish text for CodeMirror's built-in panels (search / replace / go to line). */
const ES: Record<string, string> = {
  Find: 'Buscar',
  Replace: 'Reemplazar',
  next: 'siguiente',
  previous: 'anterior',
  all: 'todos',
  'match case': 'coincidir mayúsculas',
  'by word': 'palabra completa',
  regexp: 'regex',
  replace: 'reemplazar',
  'replace all': 'reemplazar todo',
  close: 'cerrar',
  'Go to line': 'Ir a la línea',
  go: 'ir',
};

/** Phrase table for `EditorState.phrases`; empty for English (CodeMirror's own text). */
export function codemirrorPhrases(locale: Locale): Record<string, string> {
  return locale === 'es' ? ES : {};
}
