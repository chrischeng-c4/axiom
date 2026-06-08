// Utility functions — target for namespace imports (R12: import * as Utils).

import type { Todo, Filter } from "./types";

export function formatCount(count: number): string {
  return `${count} item${count !== 1 ? "s" : ""} left`;
}

export function filterTodos(todos: Todo[], filter: Filter): Todo[] {
  switch (filter) {
    case "active":
      return todos.filter((t) => !t.done);
    case "completed":
      return todos.filter((t) => t.done);
    default:
      return todos;
  }
}

export function formatDate(date: Date): string {
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

export function pluralize(count: number, singular: string, plural?: string): string {
  return count === 1 ? singular : (plural ?? singular + "s");
}
