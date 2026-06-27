// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-tests-fixtures-dom-production-build-react-bench-src-components-todolist-tsx" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import { useState, useRef } from "react";

interface Todo {
  id: number;
  text: string;
  done: boolean;
}

export function TodoList() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const nextId = useRef(1);

  const addTodo = () => {
    const text = inputRef.current?.value.trim();
    if (!text) return;
    setTodos((prev) => [...prev, { id: nextId.current++, text, done: false }]);
    inputRef.current!.value = "";
  };

  const toggle = (id: number) => {
    setTodos((prev) =>
      prev.map((t) => (t.id === id ? { ...t, done: !t.done } : t))
    );
  };

  const remove = (id: number) => {
    setTodos((prev) => prev.filter((t) => t.id !== id));
  };

  return (
    <div>
      <h2>Todos ({todos.filter((t) => !t.done).length} remaining)</h2>
      <div>
        <input ref={inputRef} placeholder="Add todo..." onKeyDown={(e) => e.key === "Enter" && addTodo()} />
        <button onClick={addTodo}>Add</button>
      </div>
      <ul>
        {todos.map((todo) => (
          <li key={todo.id} style={{ textDecoration: todo.done ? "line-through" : "none" }}>
            <input type="checkbox" checked={todo.done} onChange={() => toggle(todo.id)} />
            {todo.text}
            <button onClick={() => remove(todo.id)}>×</button>
          </li>
        ))}
      </ul>
    </div>
  );
}

// </HANDWRITE>
