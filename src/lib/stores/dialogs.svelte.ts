type DialogPayload =
  | { type: 'category'; mode: 'add'; parentKey?: string }
  | { type: 'category'; mode: 'edit'; key: string; parentKey?: string }
  | { type: 'project'; mode: 'add'; categoryKey?: string; subcategoryKey?: string }
  | { type: 'project'; mode: 'edit'; key: string }
  | { type: 'preferences' };

function createDialogStore() {
  let current = $state<DialogPayload | null>(null);

  function open(payload: DialogPayload) {
    current = payload;
  }

  function close() {
    current = null;
  }

  return {
    get current() {
      return current;
    },
    open,
    close,
  };
}

export const dialogStore = createDialogStore();
