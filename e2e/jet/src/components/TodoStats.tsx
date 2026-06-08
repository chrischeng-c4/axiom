/** @jsxRuntime classic */
/** @jsx createElement */
import { createElement } from "../mini-react";
import { percentage, PI } from "../lib"; // Deep chain: TodoStats → lib/index → lib/math
import { progressText } from "../lib/formatting"; // Direct import with aliased dep
import type { Todo } from "../types";
import { Priority } from "../types"; // Enum usage

interface TodoStatsProps {
  todos: Todo[];
}

// Tests: deep import chain via star re-export, enum, optional chaining,
// nullish coalescing, inline styles, computed values.
export function TodoStats({ todos }: TodoStatsProps) {
  const total = todos.length;
  const done = todos.filter((t: Todo) => t.done).length;
  const pct = percentage(done, total);

  // Inline style object (R: object expression in JSX prop)
  const barStyle = {
    width: `${pct}%`,
    backgroundColor: pct === 100 ? "#4caf50" : "#2196f3",
    height: "4px",
    transition: "width 0.3s",
  };

  // Optional chaining + nullish coalescing (R: modern JS syntax)
  const firstTodo = todos[0];
  const firstText = firstTodo?.text ?? "No todos yet";

  // Enum usage
  const defaultPriority = Priority.Medium;

  return (
    <div className="todo-stats" data-testid="todo-stats">
      <div className="progress-bar" data-testid="progress-bar">
        <div style={barStyle} data-testid="progress-fill"></div>
      </div>
      <span data-testid="progress-text">{progressText(done, total)}</span>
      <span data-testid="first-todo-text">{firstText}</span>
      <span data-testid="pi-value">{String(PI.toFixed(2))}</span>
      <span data-testid="priority-value">{String(defaultPriority)}</span>
    </div>
  );
}
