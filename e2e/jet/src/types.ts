// Shared types — exercises generics, enums, interfaces, utility types.
// Also tests default + named exports from the same file (R11).

export interface Todo {
  id: number;
  text: string;
  done: boolean;
}

export enum Priority {
  Low,
  Medium,
  High,
}

export interface EnhancedTodo extends Todo {
  priority: Priority;
}

export type Filter = "all" | "active" | "completed";

export type Result<T> = { data: T; error?: string };

export type TodoSummary = Pick<Todo, "id" | "text">;

export type PartialTodo = Partial<Todo>;

export type TodoWithoutId = Omit<Todo, "id">;

// Default export (R11: default + named exports from same file)
const VERSION = "1.0.0";
export default VERSION;
