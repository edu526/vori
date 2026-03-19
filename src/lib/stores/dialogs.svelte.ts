type DialogPayload =
  | { type: 'category'; mode: 'add'; parentKey?: string }
  | { type: 'category'; mode: 'edit'; key: string; parentKey?: string }
  | { type: 'project'; mode: 'add'; categoryKey?: string; subcategoryKey?: string }
  | { type: 'project'; mode: 'edit'; key: string }
  | { type: 'file'; mode: 'add' }
  | { type: 'file'; mode: 'edit'; key: string }
  | { type: 'preferences' };

function createDialogStore() {
  let current = $state<DialogPayload | null>(null);
  const escapeStack: (() => void)[] = [];

  function open(payload: DialogPayload) {
    current = payload;
  }

  function close() {
    current = null;
  }

  /** Register a sub-modal escape handler. Returns a cleanup fn to deregister. */
  function pushEscape(handler: () => void) {
    escapeStack.push(handler);
    return () => {
      const i = escapeStack.lastIndexOf(handler);
      if (i !== -1) escapeStack.splice(i, 1);
    };
  }

  /** Called by the global keydown handler on Escape. Returns true if handled. */
  function handleEscape(): boolean {
    if (escapeStack.length > 0) {
      escapeStack[escapeStack.length - 1]();
      return true;
    }
    if (current) {
      current = null;
      return true;
    }
    return false;
  }

  return {
    get current() { return current; },
    open,
    close,
    pushEscape,
    handleEscape,
  };
}

export const dialogStore = createDialogStore();
