/**
 * Mock for @tauri-apps/plugin-dialog when running in browser (npm run dev).
 */

export async function ask(message: string, options?: { title?: string }): Promise<boolean> {
  const title = options?.title ?? 'Confirm';
  return window.confirm(`${title}\n\n${message}`);
}

export async function open(options?: {
  title?: string;
  directory?: boolean;
  multiple?: boolean;
  filters?: { name: string; extensions: string[] }[];
}): Promise<string | string[] | null> {
  const msg = options?.directory
    ? 'Enter a folder path (mock):'
    : 'Enter a file path (mock):';
  const result = window.prompt(msg, '/home/user/projects/my-project');
  if (!result) return null;
  return options?.multiple ? [result] : result;
}
