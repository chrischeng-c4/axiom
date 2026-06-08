/** @jsxRuntime classic */
/** @jsx createElement */
import { createElement, useState } from "../mini-react";
import {
  APP_DISPLAY_NAME,
  STATUS_MAP,
  createConfig,
  fetchItems,
} from "../lib";

interface AppInfoProps {
  todoCount: number;
}

// Tests: aliased re-export, computed property, destructuring defaults,
// async handler, JSX in variable assignment, self-closing ternary.
export function AppInfo({ todoCount }: AppInfoProps) {
  const [asyncResult, setAsyncResult] = useState("");
  const config = createConfig(); // uses destructuring defaults

  const handleAsyncTest = () => {
    fetchItems(["a", "b", "c"]).then((items: string[]) => {
      setAsyncResult((_: string) => items.join(","));
    });
  };

  // JSX in variable assignment
  const statusBadge = STATUS_MAP.status === "ok" ? (
    <span className="status-ok" data-testid="status-badge">OK</span>
  ) : (
    <span className="status-err">ERR</span>
  );

  return (
    <div className="app-info" data-testid="app-info">
      <span data-testid="app-name">{APP_DISPLAY_NAME}</span>
      {statusBadge}
      <span data-testid="config-theme">{config.theme}</span>
      <span data-testid="config-lang">{config.lang}</span>
      <span data-testid="todo-summary">
        {todoCount > 0 ? `${todoCount} todos` : "empty"}
      </span>
      <button data-testid="async-test-btn" onClick={handleAsyncTest}>
        Test Async
      </button>
      <span data-testid="async-result">{asyncResult}</span>
    </div>
  );
}
