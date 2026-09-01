# Plan — Editor in-app + helper elevado (cross-OS)

> **Estado**: en progreso. Marcá cada item con `[x]` al completarlo.
> **Última actualización**: ver git log del archivo.

## Decisiones

| Tema | Decisión | Razón |
|---|---|---|
| Editor lib | CodeMirror 6 vía `codemirror` + `@codemirror/lang-*` | Modular, ~200KB, mount imperativo funciona con Svelte 5 runes |
| Lenguajes | markdown, json, yaml, javascript/typescript | Cubre los linked-files típicos; otros → sin highlighting |
| Elevated helper | **Re-exec del propio binario** con flag `--vori-write` | Vori detecta el flag en `main.rs` antes de iniciar Tauri; hace el write y exit. Mismo resultado que un helper externo, sin [[bin]], sin externalBin, sin plugin-shell, sin build script |
| Llamada elevated | `write_text_file_elevated` intenta directo; fallback re-exec | 90% de archivos no necesita prompt |
| Disparador | Ambos — auto-detect por extensión + botón explícito | Pedido del usuario |
| UI del editor | **Split view en ColumnBrowser** (columna del file a la izquierda, editor a la derecha, animado) | La ventana tiene minWidth 960px → nunca se ve apretado. Animación: columnas a la izquierda colapsan con translateX(-100%), la columna del file queda visible, el panel editor se expande desde la derecha. El modal EditorDialog queda solo como fallback para HomeView/RecentsView si fuera necesario, pero con la columna siempre visible en +page.svelte el split funciona siempre |
| Dirty state | Save deshabilitado hasta editar; cerrar con cambios → confirm | UX estándar |

---

## Phase 1 — Backend: IPC básico de read/write

- [x] Crear `src-tauri/src/commands/files_io.rs` con `read_text_file` (cap 5MB)
- [x] Agregar `write_text_file` al mismo archivo
- [x] Re-exportar módulo en `src-tauri/src/commands/mod.rs`
- [x] Importar y registrar comandos en `src-tauri/src/lib.rs` `invoke_handler!`
- [x] `cargo check --manifest-path src-tauri/Cargo.toml --bin vori` pasa

## Phase 2 — Re-exec del propio binario (helper mode)

- [x] Detectar flag `--vori-write <target> <temp>` en `src-tauri/src/main.rs` antes de `vori_lib::run()`
- [x] Implementar `vori_write_main` (lee temp, escribe target, borra temp, exit code)
- [x] `cargo check --manifest-path src-tauri/Cargo.toml` pasa

## Phase 3 — `write_text_file_elevated` con elevación por OS

- [x] Agregar `write_text_file_elevated` a `files_io.rs` (intenta directo, fallback re-exec elevado)
- [x] Branch Windows en helper de elevación (PowerShell Start-Process -Verb RunAs)
- [x] Branch macOS (osascript with administrator privileges)
- [x] Branch Linux (pkexec, fallback sudo -n)
- [x] Registrar en `commands/mod.rs` y `lib.rs` `invoke_handler!`
- [x] `cargo check` pasa

## Phase 4 — ~~Build script + packaging~~ → cancelado (no aplica con re-exec)

## Phase 5 — Frontend deps

- [x] Instalar `codemirror`, `@codemirror/lang-markdown/json/yaml/javascript`, `@codemirror/theme-one-dark`
- [x] `pnpm check` pasa después de instalar

## Phase 6 — Editor dialog + extension detector

- [x] Crear `src/lib/utils/textExtensions.ts` con set de extensiones + `isTextFile()` + `languageForPath()`
- [x] Agregar variant `{ type: 'editor'; filePath; fileName }` a `src/lib/stores/dialogs.svelte.ts`
- [x] Agregar wrappers `readTextFile`, `writeTextFile`, `writeTextFileElevated` en `src/lib/api/commands.ts`
- [x] Crear `src/lib/components/dialogs/EditorDialog.svelte` (CodeMirror mount + save con fallback admin + cancel dirty check)
- [x] `pnpm check` pasa

## Phase 7 — Integración UI

- [x] Agregar item "Edit in Vori" en `src/lib/components/context-menu/menuBuilder.ts` (case 'file', solo si isTextFile)
- [x] Agregar prop `onedit` y botón de lápiz en hover en `src/lib/components/columns/ColumnItem.svelte` (solo si isTextFile)
- [x] Pasar `onedit` desde `Column.svelte` al item
- [x] Wire `onedit` handler en `src/lib/components/columns/ColumnBrowser.svelte` (abre `dialogStore.open({ type: 'editor', ... })`)
- [x] Agregar auto-detect en `handleOpen` de `ColumnBrowser.svelte` (text ext → editor, sino externo)
- [x] Mismo auto-detect en `src/lib/components/views/HomeView.svelte`, `RecentsView.svelte`, `routes/+page.svelte`
- [x] `pnpm check` pasa

## Phase 8 — Smoke test full

- [ ] `pnpm tauri dev` arranca sin errores
- [ ] Add `.txt` file → dblclick → editor abre, editar, save → archivo cambia en disco
- [ ] Add `.png` file → dblclick → abre con app externa (no editor)
- [ ] Right-click file → "Edit in Vori" → editor abre
- [ ] Edit archivo en `C:\Windows\System32\drivers\etc\hosts` → save → permission error → "¿admin?" → sí → UAC → write OK
- [ ] Mismo flow en macOS con `/etc/hosts`
- [ ] Mismo flow en Linux con archivo root-only

## Phase 9 — UI: split view con animación en ColumnBrowser

- [x] Crear `src/lib/components/editor/EditorPane.svelte` (CodeMirror + save/cancel/close, extraído de EditorDialog)
- [x] Modificar `ColumnBrowser.svelte`: detectar `isEditing` desde `dialogStore`, filtrar columnas, render split view
- [x] CSS: animación de columnas (translateX + opacity) + flex-basis del editor pane
- [x] `+page.svelte`: ocultar `HomeView` cuando el editor está activo
- [x] Quitar `EditorDialog` del conditional render en `+page.svelte` (ya no se usa)
- [x] `pnpm check` pasa

## Phase 10 — Bug fixes del editor

- [x] **No recrear editor en cada keystroke**: usar `untrack` para leer `content` y `configStore.preferences` en el mount effect; guardar `mountedFor` para evitar re-mount
- [x] **Save no debe cerrar el editor**: quitar `onclose()` de save; agregar indicador "✓ Guardado" por 1.5s
- [x] **Layout sin columna vacía**: `column-browser` con `flex: 0 0 auto` por default, `flex: 1` solo cuando `multi` (multi = !isSingleCol || isEditing)

## Phase 11 — Opciones de editor configurables

- [x] Agregar `editor_text_wrap`, `editor_tab_size`, `editor_font_size` al modelo Rust Preferences con serde defaults
- [x] Agregar mismas campos al interface TypeScript Preferences
- [x] UI en PreferencesDialog → tab Editors → sección "In-app editor" (checkbox wrap, select tab size, botones +/- font size)
- [x] EditorPane lee configStore.preferences y aplica (indentUnit, lineWrapping, fontSize)
- [x] Instalar `@codemirror/language` (para indentUnit)
- [x] Actualizar stores/mocks con nuevos defaults
- [x] `pnpm check` y `cargo check` pasan

## Phase 12 — Confirm al cambiar de archivo con cambios sin guardar

- [x] Crear `src/lib/stores/editor.svelte.ts` (global dirty + currentFilePath)
- [x] Crear `src/lib/utils/openEditor.ts` (helper que checkea dirty antes de abrir el editor)
- [x] EditorPane sincroniza su `dirty` y `filePath` al editorStore; limpia en onDestroy
- [x] Reemplazar `dialogStore.open({type:'editor',...})` por `openEditor(...)` en: ColumnBrowser (handleOpen, handleEdit), HomeView, RecentsView, +page.svelte, menuBuilder
- [x] `pnpm check` pasa

## Phase 13 — Bug fixes adicionales

- [x] **Undo no debe marcar dirty**: nueva variable `savedDoc` = último contenido guardado/cargado; `dirty = content !== savedDoc` (en vez de `dirty=true` en cualquier docChange)
- [x] **Click en categoría durante edit**: cierra el editor primero antes de navegar (sino el slice filter oculta la nueva columna)
- [x] **Focus del editor**: múltiples intentos de focus (immediate + rAF + 50ms + 200ms) + onclick en .pane-body para refocus

---

## Gotchas (no son tareas, son trampas a evitar)

1. **Re-exec y stdin**: el proceso elevado (vía UAC/sudo/osascript) NO hereda stdin — por eso pasamos content por temp file, no por stdin.
2. **PowerShell escaping** (Windows): comillas simples en paths requieren duplicarse (`'` → `''`); usar `ps_quote()`.
3. **macOS shell escape**: dentro de `do shell script`, tanto `\` como `"` requieren escape. Backticks invertidos no son problema aquí.
4. **Linux sin polkit**: si `pkexec` y `sudo -n` fallan, devolver mensaje claro ("instalá pkexec/sudo o ejecutá Vori como root"), no crashear.
5. **Cap de 5 MB** en `read_text_file` para no freezar la UI.
6. **CodeMirror cleanup**: el `$effect` debe retornar cleanup `() => view.destroy()`. Sin esto, cada apertura del editor acumula listeners y fugas.
7. **Dialog re-mount**: EditorDialog monta CodeMirror cada vez que `isOpen` pasa a true; dispose en cleanup evita leaks de DOM.
8. **`windows_subsystem = "windows"` en dev**: en debug builds el proceso elevado muestra una ventana de consola brevemente. Solo aplica a debug; release es invisible.
9. **Outer button → div role=button**: ColumnItem es `<div>` (no `<button>`) para permitir el botón "✎" anidado. HTML prohíbe button-in-button.
10. **Auto-detect scope**: `isTextFile()` solo aplica a text files. Context menu y botón hover se ocultan para binarios — abrir el editor para un `.png` no tiene sentido.