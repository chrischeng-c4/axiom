// Tests aliased imports: import { X as Y }

import { percentage as calcPct } from "./math";

export function progressText(done: number, total: number): string {
  const pct = calcPct(done, total);
  return `${pct}% complete`;
}
