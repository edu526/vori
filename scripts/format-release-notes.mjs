// Turns a semantic-release changelog section into GitHub release notes.
//
// The section starts with a heading like `# [1.7.0](<compare url>) (2026-09-21)`.
// GitHub already shows the version (title/tag) and the date, so drop the heading
// and keep its compare link as a trailing "Full Changelog" line.
//
// Usage: node scripts/format-release-notes.mjs < notes.md > formatted.md
// Idempotent: input without such a heading is passed through unchanged.
import { readFileSync } from 'fs';

const lines = readFileSync(0, 'utf8').replace(/\r\n/g, '\n').split('\n');

const first = lines.findIndex((l) => l.trim() !== '');
const heading = /^#{1,2}\s+\[?\d+\.\d+\.\d+\]?/;

let out = lines;
let compareUrl = null;
if (first !== -1 && heading.test(lines[first])) {
  compareUrl = lines[first].match(/\((https:\/\/[^)\s]+)\)/)?.[1] ?? null;
  out = lines.slice(first + 1);
}

let body = out.join('\n').trim();
if (compareUrl) body += `\n\n**Full Changelog**: ${compareUrl}`;
process.stdout.write(body + '\n');
