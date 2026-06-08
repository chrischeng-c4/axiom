/** @jsxRuntime classic */
/** @jsx createElement */
import { createElement, useState } from "./mini-react";
import { Header, TodoItem, TodoFooter, TodoStats, AppInfo } from "./components";
import { useLocalStorage } from "./hooks/useLocalStorage";
import * as Utils from "./utils";
import type { Todo, Filter, Result } from "./types";
import VERSION from "./types";

let nextId = 1;

export function App() {
  const [todos, setTodos] = useLocalStorage<Todo[]>("mini-react-todos", []);
  const [input, setInput] = useState("");
  const [filter, setFilter] = useState<Filter>("all");
  const [showAbout, setShowAbout] = useState(false);
  const [aboutContent, setAboutContent] = useState<Result<string>>({ data: "" });
  const [showSettings, setShowSettings] = useState(false);
  const [settingsLoaded, setSettingsLoaded] = useState(false);

  const addTodo = () => {
    const text = input.trim();
    if (!text) return;
    setTodos((prev: Todo[]) => [...prev, { id: nextId++, text, done: false }]);
    setInput("");
  };

  const toggleTodo = (id: number) => {
    setTodos((prev: Todo[]) =>
      prev.map((t: Todo) => (t.id === id ? { ...t, done: !t.done } : t))
    );
  };

  const removeTodo = (id: number) => {
    setTodos((prev: Todo[]) => prev.filter((t: Todo) => t.id !== id));
  };

  const toggleAll = () => {
    const allDone = todos.every((t: Todo) => t.done);
    setTodos((prev: Todo[]) => prev.map((t: Todo) => ({ ...t, done: !allDone })));
  };

  const clearCompleted = () => {
    setTodos((prev: Todo[]) => prev.filter((t: Todo) => !t.done));
  };

  const loadAbout = () => {
    setShowAbout((prev: boolean) => !prev);
    if (!showAbout) {
      // Dynamic import — code splitting boundary (R6)
      import("./pages/About").then(() => {
        setAboutContent((_: Result<string>) => ({ data: "loaded" }));
      });
    }
  };

  const loadSettings = () => {
    setShowSettings((prev: boolean) => !prev);
    if (!showSettings) {
      // Second dynamic import — tests multiple lazy loads
      import("./pages/Settings").then(() => {
        setSettingsLoaded((_: boolean) => true);
      });
    }
  };

  const filtered = Utils.filterTodos(todos, filter);

  return (
    <div className="todoapp" data-testid="app">
      <Header input={input} onInput={setInput} onAdd={addTodo} />

      <div className="version-info" data-testid="version">
        v{VERSION}
      </div>

      {todos.length > 0 ? (
        <section className="main">
          <button
            className="toggle-all"
            data-testid="toggle-all"
            onClick={toggleAll}
          >
            Toggle All
          </button>

          <ul className="todo-list" data-testid="todo-list">
            {filtered.map((todo: Todo) => (
              <TodoItem
                {...todo}
                onToggle={toggleTodo}
                onRemove={removeTodo}
              />
            ))}
          </ul>

          <TodoFooter
            todos={todos}
            filter={filter}
            onFilterChange={setFilter}
            onClearCompleted={clearCompleted}
          />

          <TodoStats todos={todos} />
        </section>
      ) : (
        ""
      )}

      <AppInfo todoCount={todos.length} />

      <button data-testid="about-btn" onClick={loadAbout}>
        {showAbout ? "Hide About" : "Show About"}
      </button>
      {showAbout && aboutContent.data ? (
        <div className="about-section" data-testid="about-section">
          <p>About page loaded successfully.</p>
          <p>Created: {Utils.formatDate(new Date())}</p>
        </div>
      ) : (
        ""
      )}

      <button data-testid="settings-btn" onClick={loadSettings}>
        {showSettings ? "Hide Settings" : "Show Settings"}
      </button>
      {showSettings && settingsLoaded ? (
        <div className="settings-section" data-testid="settings-section">
          <p>Settings loaded.</p>
        </div>
      ) : (
        ""
      )}
    </div>
  );
}
