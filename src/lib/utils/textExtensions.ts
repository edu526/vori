import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import { markdown } from '@codemirror/lang-markdown';
import { yaml } from '@codemirror/lang-yaml';
import type { Extension } from '@codemirror/state';

const TEXT_EXTENSIONS = new Set([
  'txt', 'md', 'markdown', 'log',
  'json', 'jsonc', 'json5',
  'yaml', 'yml', 'toml',
  'ini', 'conf', 'cfg', 'config', 'env',
  'xml', 'html', 'htm', 'css', 'scss', 'less',
  'js', 'jsx', 'mjs', 'cjs',
  'ts', 'tsx', 'mts', 'cts',
  'py', 'rb', 'php', 'java', 'kt', 'swift', 'go', 'rs',
  'sh', 'bash', 'zsh', 'fish', 'ps1',
  'sql', 'graphql', 'gql',
  'csv', 'tsv',
  'gitignore', 'dockerignore', 'editorconfig',
]);

export function isTextFile(path: string): boolean {
  const name = path.split(/[\\/]/).pop() ?? '';
  const ext = name.includes('.') ? name.split('.').pop()!.toLowerCase() : '';
  if (!ext) return true;
  return TEXT_EXTENSIONS.has(ext);
}

export function languageForPath(path: string): Extension {
  const name = path.split(/[\\/]/).pop() ?? '';
  const ext = name.includes('.') ? name.split('.').pop()!.toLowerCase() : '';
  switch (ext) {
    case 'md':
    case 'markdown':
      return markdown();
    case 'json':
    case 'jsonc':
      return json();
    case 'yaml':
    case 'yml':
      return yaml();
    case 'js':
    case 'jsx':
    case 'mjs':
    case 'cjs':
    case 'ts':
    case 'tsx':
    case 'mts':
    case 'cts':
      return javascript({ jsx: true, typescript: true });
    default:
      return [];
  }
}