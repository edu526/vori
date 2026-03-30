import { readFileSync, writeFileSync } from 'fs';

const version = process.argv[2];
if (!version) {
  console.error('Usage: bump-version.mjs <version>');
  process.exit(1);
}

// package.json
const pkg = JSON.parse(readFileSync('package.json', 'utf8'));
pkg.version = version;
writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');

// tauri.conf.json
const tauri = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
tauri.version = version;
writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(tauri, null, 2) + '\n');

// Cargo.toml — only replace the first version = "..." (the package version)
let cargo = readFileSync('src-tauri/Cargo.toml', 'utf8');
cargo = cargo.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);
writeFileSync('src-tauri/Cargo.toml', cargo);

console.log(`Bumped to v${version}`);
