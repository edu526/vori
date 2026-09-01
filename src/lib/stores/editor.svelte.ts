function createEditorStore() {
  let dirty = $state(false);
  let currentFilePath = $state<string | null>(null);

  return {
    get dirty() {
      return dirty;
    },
    get currentFilePath() {
      return currentFilePath;
    },
    setDirty(v: boolean) {
      dirty = v;
    },
    setCurrentFilePath(v: string | null) {
      currentFilePath = v;
    },
  };
}

export const editorStore = createEditorStore();