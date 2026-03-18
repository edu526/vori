export interface MenuItem {
  label: string;
  action: () => void;
  danger?: boolean;
  disabled?: boolean;
  divider?: boolean;
}

function createContextMenuStore() {
  let visible = $state(false);
  let x = $state(0);
  let y = $state(0);
  let items = $state<MenuItem[]>([]);

  function show(newX: number, newY: number, newItems: MenuItem[]) {
    x = newX;
    y = newY;
    items = newItems;
    visible = true;
  }

  function hide() {
    visible = false;
  }

  return {
    get visible() { return visible; },
    get x() { return x; },
    get y() { return y; },
    get items() { return items; },
    show,
    hide,
  };
}

export const contextMenuStore = createContextMenuStore();
