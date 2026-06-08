let counter = 0;

export function generateId(): string {
  counter += 1;
  const ts = Date.now().toString(36);
  const seq = counter.toString(36);
  return `${ts}-${seq}`;
}
