import { useState } from "react";
import { Counter } from "./components/Counter";
import { TodoList } from "./components/TodoList";

export function App() {
  const [page, setPage] = useState<"counter" | "todos">("counter");

  return (
    <div style={{ maxWidth: 600, margin: "0 auto", padding: 20 }}>
      <h1>React Bench</h1>
      <nav style={{ marginBottom: 20 }}>
        <button
          onClick={() => setPage("counter")}
          style={{ fontWeight: page === "counter" ? "bold" : "normal" }}
        >
          Counter
        </button>
        {" | "}
        <button
          onClick={() => setPage("todos")}
          style={{ fontWeight: page === "todos" ? "bold" : "normal" }}
        >
          Todos
        </button>
      </nav>
      {page === "counter" ? <Counter /> : <TodoList />}
    </div>
  );
}
