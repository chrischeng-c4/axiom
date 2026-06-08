// Pure utility — tests export const + export function
// Imported via star re-export barrel (lib/index.ts)

export const PI = 3.14159;

export function clamp(val: number, min: number, max: number): number {
  return Math.min(Math.max(val, min), max);
}

export function percentage(done: number, total: number): number {
  return total === 0 ? 0 : Math.round((done / total) * 100);
}
