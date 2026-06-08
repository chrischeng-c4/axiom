// Tests: async/await, try/catch, for...of loops.

export async function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function fetchItems<T>(items: T[]): Promise<T[]> {
  await delay(0);
  const result: T[] = [];
  for (const item of items) {
    result.push(item);
  }
  return result;
}

export async function safeParse(json: string): Promise<unknown> {
  try {
    return JSON.parse(json);
  } catch {
    return null;
  }
}
