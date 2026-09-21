import { t } from '$lib/i18n/index.svelte';

/**
 * Turns release notes (the markdown of a GitHub release) into short plain text for a
 * native dialog: no links, no commit hashes, no markdown syntax.
 */
export function notesToPlainText(body: string | undefined | null, maxLines = 12): string {
  if (!body) return '';

  const lines: string[] = [];
  for (const raw of body.replace(/\r\n/g, '\n').split('\n')) {
    let line = raw.trim();
    if (!line || /^\*\*Full Changelog\*\*/i.test(line)) continue;

    line = line
      .replace(/\s*\(\[[0-9a-f]{7,40}\]\([^)]*\)\)/g, '') // trailing "([abc1234](url))"
      .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1') // [text](url) -> text
      .replace(/\*\*/g, '');

    const heading = line.match(/^#{1,6}\s+(.*)$/);
    if (heading) {
      lines.push(`${heading[1]}:`);
      continue;
    }
    lines.push(line.replace(/^[*-]\s+/, '• '));
  }

  if (lines.length <= maxLines) return lines.join('\n');
  return [...lines.slice(0, maxLines), t('update.more', { count: lines.length - maxLines })].join('\n');
}
