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
  "@tauri-apps/api/webviewWindow": mock("tauri-webviewwindow.ts"),
  "@tauri-apps/api/dpi": mock("tauri-dpi.ts"),
  "@tauri-apps/plugin-dialog": mock("tauri-dialog.ts"),
};

/**
 * Wrap @tailwindcss/vite plugins to exclude Svelte virtual style modules
 * from Tailwind's CSS transform pipeline.
 *
 * WHY: Tailwind tries to parse every file matching /&lang\.css/ as CSS.
 * Svelte virtual style modules (?svelte&type=style&lang.css) sometimes have
 * JS code bleeding in, causing "Invalid declaration" errors. Tailwind doesn't
 * need to transform these modules — it detects utility classes by scanning
 * the raw .svelte files directly via its Oxide scanner. Excluding the virtual
 * modules from the CSS transform is the clean fix.
 */
function tailwindExcludeSvelte() {
  const plugins = tailwindcss();
  const arr = Array.isArray(plugins) ? plugins : [plugins];
  return arr.map((p) => {
    if (!p?.transform?.filter?.id) return p;
    return {
      ...p,
      transform: {
        ...p.transform,
        filter: {
          ...p.transform.filter,
          id: {
            ...p.transform.filter.id,
            exclude: [
              ...(p.transform.filter.id.exclude ?? []),
              /[?&]svelte/, // exclude all Svelte virtual modules
            ],
          },
        },
      },
    };
  });
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    sveltekit(),
    ...tailwindExcludeSvelte(),
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
