// Custom hook: persist state to localStorage (R2).

import { useState } from "../mini-react";

export function useLocalStorage<T>(key: string, initialValue: T): [T, (v: T | ((prev: T) => T)) => void] {
  const stored = localStorage.getItem(key);
  const initial: T = stored ? JSON.parse(stored) : initialValue;

  const [value, setValue] = useState<T>(initial);

  const setAndStore = (v: T | ((prev: T) => T)) => {
    setValue((prev: T) => {
      const next = typeof v === "function" ? (v as (prev: T) => T)(prev) : v;
      localStorage.setItem(key, JSON.stringify(next));
      return next;
    });
  };

  return [value, setAndStore];
}
