## [1.2.1](https://github.com/edu526/vori/compare/v1.2.0...v1.2.1) (2026-06-24)


### Bug Fixes

* **ci:** pass --repo to gh release edit so it runs without git context ([2c95122](https://github.com/edu526/vori/commit/2c951227048662e1702987bf26acc97441ff0f8d))

# [1.2.0](https://github.com/edu526/vori/compare/v1.1.1...v1.2.0) (2026-06-24)


### Bug Fixes

* center window on initial launch ([e3200dd](https://github.com/edu526/vori/commit/e3200dd6cb9441b554e157914cc8b182c248810a))


### Features

* add in-app updater with signed GitHub releases ([7936489](https://github.com/edu526/vori/commit/7936489a18785950a833707944a3a3287f63504c))

## [1.1.1](https://github.com/edu526/vori/compare/v1.1.0...v1.1.1) (2026-06-23)


### Bug Fixes

* **ci:** pass --target before -- in tauri build so CLI tracks output path ([007f466](https://github.com/edu526/vori/commit/007f466bbb5a88ae56832aaa9b80107f3cb30a3a))

# [1.1.0](https://github.com/edu526/vori/compare/v1.0.0...v1.1.0) (2026-06-23)


### Bug Fixes

* **ci:** drop duplicate pnpm version in release workflow ([ab42b00](https://github.com/edu526/vori/commit/ab42b00045d644dea6e08765cd4e0ab50c6e3037))
* split filesystem paths on both slash directions so project name shows on Windows ([0563f02](https://github.com/edu526/vori/commit/0563f02816e62eafd37b850cb2eef0df61d4a72e))


### Features

* implement project launcher and terminal/editor detection services with global preference management ([0580399](https://github.com/edu526/vori/commit/058039922563db5e89113efd1cb99d8153a942c6))

# 1.0.0 (2026-03-30)


### Bug Fixes

* block native browser context menu globally ([3bd231b](https://github.com/edu526/vori/commit/3bd231b8e0e98ca476d068e73867a38dbe1c78c4))
* cleaner $effect for column auto-scroll ([a1d3e0b](https://github.com/edu526/vori/commit/a1d3e0bc11d6c2dbf5f6b33a4d9de8869bb8e243))
* **dialogs:** support subcategory edit and fix all a11y warnings ([b87116c](https://github.com/edu526/vori/commit/b87116c88f97c8515a2da8b5e926744d1cf0215e))
* favorites __r: key bug, auto-scroll columns, Ctrl+F search focus ([7d1c674](https://github.com/edu526/vori/commit/7d1c674a4d0f9447ffd5ed64f466fc3038bc21e9))
* live recents and favorites sync without restart ([227688d](https://github.com/edu526/vori/commit/227688de272e99be31a079355989af9fbd6172fb))
* search files, instant favorites, explicit dialog capabilities ([5260cf5](https://github.com/edu526/vori/commit/5260cf58f7529888fb58f1ad3d652af1d0db618b))
* search ordering, subcategory header, auto-detect terminals ([f397bc9](https://github.com/edu526/vori/commit/f397bc93c6db637aa2975a1e3cd070bd24b400b8))
* sort all column items alphabetically ([5dbf258](https://github.com/edu526/vori/commit/5dbf25804af3df5ba1b53c7eb0920793c90b3446))


### Features

* add semantic icons per item type via ItemIcon component ([3076ace](https://github.com/edu526/vori/commit/3076aceff855b0248b19c1be1bf1903ab4a9567d))
* **editors:** add editor auto-detection support ([d0624a1](https://github.com/edu526/vori/commit/d0624a1c3bf67e07419964f69038053ac7261a61))
* enhance PreferencesDialog with theme selection and editor/terminal management ([d3c2000](https://github.com/edu526/vori/commit/d3c2000f597a852befd9a261661a29cad9c89d33))
* **files:** add FileDialog and fix delete confirmation + live refresh ([7d9f1e3](https://github.com/edu526/vori/commit/7d9f1e32a845c068e0dfa1a2fcbe6506ee9e52d9))
* initial Vori architecture — Tauri 2 + Svelte 5 + Rust backend ([b2dffba](https://github.com/edu526/vori/commit/b2dffba19017169f757d34007aa1c04976a6f42b))
* Introduce `HomeView` component to display favorite and recent items, replacing `RecentsView`, and remove deprecated Linux desktop template configuration. ([44784b7](https://github.com/edu526/vori/commit/44784b77b22d3e2e55eadb49bb56287bf8b8853b))
* keyboard navigation + empty state onboarding ([a0ea56e](https://github.com/edu526/vori/commit/a0ea56ea435dbbdc2416002d3e44742b1654d4c7))
* migrate dialogs to shadcn and add browser dev mocks ([d63130d](https://github.com/edu526/vori/commit/d63130d124873cafc698837f741fdff259a66d05))
* recents section, favorites badges, close_on_open, terminal fix ([056c3e8](https://github.com/edu526/vori/commit/056c3e8f8537e55b0708b04ffd9d94e0a5684f9c))
* replace recents in root column with RecentsView panel ([88cdb5d](https://github.com/edu526/vori/commit/88cdb5ddd4783f928f20ed80f3b4f6ea99884d80))
* **search:** keyboard navigation for search results ([5f27874](https://github.com/edu526/vori/commit/5f278746724f927c3a48b686fb73cd96ca89d0a3))
* show app context menu on right-click in empty column areas ([75bbe1c](https://github.com/edu526/vori/commit/75bbe1c25ae7f3995561f93c2fb74a12538648d3))
* single instance via tauri-plugin-single-instance ([165ae4e](https://github.com/edu526/vori/commit/165ae4ed37f373c70e6311754ece1666acb38993))
* **tauri:** add tray icon toggle, window management, and preference controls ([ab469de](https://github.com/edu526/vori/commit/ab469de9743cf3b3b796ae1c1f1d17ad2c352d51))
* **ui:** multi-project workspace selection and distinct project icon ([bc7a375](https://github.com/edu526/vori/commit/bc7a375fbbe960ee1fbe4bd1f90a73fcd542a167))
* **ui:** sliding column browser, zoom shortcuts, and Tailwind/Svelte fix ([aff5c60](https://github.com/edu526/vori/commit/aff5c6064dde24c3080b99c00412154f2366d24e))
* **ui:** spotlight search modal, custom titlebar, remove detail panel ([cbdf9cd](https://github.com/edu526/vori/commit/cbdf9cd3586f312d5dc40b1bcd186e069377d812))

# 1.0.0 (2026-03-30)


### Bug Fixes

* block native browser context menu globally ([3bd231b](https://github.com/edu526/vori/commit/3bd231b8e0e98ca476d068e73867a38dbe1c78c4))
* cleaner $effect for column auto-scroll ([a1d3e0b](https://github.com/edu526/vori/commit/a1d3e0bc11d6c2dbf5f6b33a4d9de8869bb8e243))
* **dialogs:** support subcategory edit and fix all a11y warnings ([b87116c](https://github.com/edu526/vori/commit/b87116c88f97c8515a2da8b5e926744d1cf0215e))
* favorites __r: key bug, auto-scroll columns, Ctrl+F search focus ([7d1c674](https://github.com/edu526/vori/commit/7d1c674a4d0f9447ffd5ed64f466fc3038bc21e9))
* live recents and favorites sync without restart ([227688d](https://github.com/edu526/vori/commit/227688de272e99be31a079355989af9fbd6172fb))
* search files, instant favorites, explicit dialog capabilities ([5260cf5](https://github.com/edu526/vori/commit/5260cf58f7529888fb58f1ad3d652af1d0db618b))
* search ordering, subcategory header, auto-detect terminals ([f397bc9](https://github.com/edu526/vori/commit/f397bc93c6db637aa2975a1e3cd070bd24b400b8))
* sort all column items alphabetically ([5dbf258](https://github.com/edu526/vori/commit/5dbf25804af3df5ba1b53c7eb0920793c90b3446))


### Features

* add semantic icons per item type via ItemIcon component ([3076ace](https://github.com/edu526/vori/commit/3076aceff855b0248b19c1be1bf1903ab4a9567d))
* **editors:** add editor auto-detection support ([d0624a1](https://github.com/edu526/vori/commit/d0624a1c3bf67e07419964f69038053ac7261a61))
* enhance PreferencesDialog with theme selection and editor/terminal management ([d3c2000](https://github.com/edu526/vori/commit/d3c2000f597a852befd9a261661a29cad9c89d33))
* **files:** add FileDialog and fix delete confirmation + live refresh ([7d9f1e3](https://github.com/edu526/vori/commit/7d9f1e32a845c068e0dfa1a2fcbe6506ee9e52d9))
* initial Vori architecture — Tauri 2 + Svelte 5 + Rust backend ([b2dffba](https://github.com/edu526/vori/commit/b2dffba19017169f757d34007aa1c04976a6f42b))
* Introduce `HomeView` component to display favorite and recent items, replacing `RecentsView`, and remove deprecated Linux desktop template configuration. ([44784b7](https://github.com/edu526/vori/commit/44784b77b22d3e2e55eadb49bb56287bf8b8853b))
* keyboard navigation + empty state onboarding ([a0ea56e](https://github.com/edu526/vori/commit/a0ea56ea435dbbdc2416002d3e44742b1654d4c7))
* migrate dialogs to shadcn and add browser dev mocks ([d63130d](https://github.com/edu526/vori/commit/d63130d124873cafc698837f741fdff259a66d05))
* recents section, favorites badges, close_on_open, terminal fix ([056c3e8](https://github.com/edu526/vori/commit/056c3e8f8537e55b0708b04ffd9d94e0a5684f9c))
* replace recents in root column with RecentsView panel ([88cdb5d](https://github.com/edu526/vori/commit/88cdb5ddd4783f928f20ed80f3b4f6ea99884d80))
* **search:** keyboard navigation for search results ([5f27874](https://github.com/edu526/vori/commit/5f278746724f927c3a48b686fb73cd96ca89d0a3))
* show app context menu on right-click in empty column areas ([75bbe1c](https://github.com/edu526/vori/commit/75bbe1c25ae7f3995561f93c2fb74a12538648d3))
* single instance via tauri-plugin-single-instance ([165ae4e](https://github.com/edu526/vori/commit/165ae4ed37f373c70e6311754ece1666acb38993))
* **tauri:** add tray icon toggle, window management, and preference controls ([ab469de](https://github.com/edu526/vori/commit/ab469de9743cf3b3b796ae1c1f1d17ad2c352d51))
* **ui:** multi-project workspace selection and distinct project icon ([bc7a375](https://github.com/edu526/vori/commit/bc7a375fbbe960ee1fbe4bd1f90a73fcd542a167))
* **ui:** sliding column browser, zoom shortcuts, and Tailwind/Svelte fix ([aff5c60](https://github.com/edu526/vori/commit/aff5c6064dde24c3080b99c00412154f2366d24e))
* **ui:** spotlight search modal, custom titlebar, remove detail panel ([cbdf9cd](https://github.com/edu526/vori/commit/cbdf9cd3586f312d5dc40b1bcd186e069377d812))
