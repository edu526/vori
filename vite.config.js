import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
// @ts-expect-error process is a nodejs global
const isTauri = !!process.env.TAURI_ENV_PLATFORM;
// @ts-expect-error process is a nodejs global
const cwd = process.cwd();

/** @param {string} file */
function mock(file) {
  return `${cwd}/src/lib/mocks/${file}`;
}

/** @type {Record<string, string>} */
const browserAlias = {
  "@tauri-apps/api/core": mock("tauri-core.ts"),
  "@tauri-apps/api/window": mock("tauri-window.ts"),
  "@tauri-apps/plugin-dialog": mock("tauri-dialog.ts"),
};

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    sveltekit(),
    // Strip JS import statements that bleed into Svelte virtual style modules.
    // @tailwindcss/vite:generate:serve tries to parse them as CSS and logs
    // "Invalid declaration" warnings for every import line it finds.
    {
      name: 'svelte-style-sanitize',
      enforce: 'pre',
      transform(code, id) {
        if (id.includes('?svelte&type=style')) {
          return { code: code.replace(/^[ \t]*import\s[^\n]+$/gm, ''), map: null };
        }
      },
    },
    tailwindcss(),
  ],

  resolve: {
    alias: isTauri ? {} : browserAlias,
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: isTauri,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
});
