/** @jsxRuntime classic */
/** @jsx createElement */
/** @jsxFrag Fragment */
import { createElement, Fragment } from "../mini-react";
import type { Todo, Filter } from "../types";
import * as Utils from "../utils";

interface TodoFooterProps {
  todos: Todo[];
  filter: Filter;
  onFilterChange: (f: Filter) => void;
  onClearCompleted: () => void;
}

// Uses fragments (R5), namespace imports (R12), conditional JSX (R4).
export function TodoFooter({ todos, filter, onFilterChange, onClearCompleted }: TodoFooterProps) {
  const remaining = todos.filter((t) => !t.done).length;
  const hasCompleted = todos.some((t) => t.done);

  return (
    <footer className="footer" data-testid="footer">
      <span className="todo-count" data-testid="count">
        {Utils.formatCount(remaining)}
      </span>
      <>
        <div className="filters" data-testid="filters">
          <button
            className={filter === "all" ? "selected" : ""}
            onClick={() => onFilterChange("all")}
          >
            All
          </button>
          <button
            className={filter === "active" ? "selected" : ""}
            onClick={() => onFilterChange("active")}
          >
            Active
          </button>
          <button
            className={filter === "completed" ? "selected" : ""}
            onClick={() => onFilterChange("completed")}
          >
            Completed
          </button>
        </div>
      </>
      {hasCompleted ? (
        <button
          className="clear-completed"
          data-testid="clear-completed"
          onClick={onClearCompleted}
        >
          Clear completed
        </button>
      ) : (
        ""
      )}
    </footer>
  );
}
