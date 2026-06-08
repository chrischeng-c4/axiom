/** @jsxRuntime classic */
/** @jsx createElement */
import { createElement } from "../mini-react";
import type { Todo } from "../types";

interface TodoItemProps extends Todo {
  onToggle: (id: number) => void;
  onRemove: (id: number) => void;
}

// Uses spread props (R3), template literals (R8), conditional JSX (R4).
export function TodoItem(props: TodoItemProps) {
  const { id, text, done, onToggle, onRemove } = props;

  return (
    <li
      className={`todo-item ${done ? "completed" : ""}`}
      data-testid={`todo-${id}`}
      data-todo-id={String(id)}
    >
      <input
        type="checkbox"
        className="toggle"
        checked={done}
        onClick={() => onToggle(id)}
      />
      <span className="todo-text">{text}</span>
      {done && <span className="done-badge">✓</span>}
      <button
        className="destroy"
        data-testid={`delete-${id}`}
        onClick={() => onRemove(id)}
      >
        x
      </button>
    </li>
  );
}
