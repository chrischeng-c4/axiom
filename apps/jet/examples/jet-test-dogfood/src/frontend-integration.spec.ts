// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-examples-jet-test-dogfood-src.md#tests
// CODEGEN-BEGIN
// Frontend-integration-style dogfood spec. Wires multiple pure modules
// together (todo reducer + selectors) without touching the DOM, proving
// the runner can host a non-trivial app slice end-to-end.

import { describe, test, expect, beforeEach } from "@jet/test";

type Todo = { id: number; text: string; done: boolean };
type State = { todos: Todo[]; nextId: number };

type Action =
  | { kind: "add"; text: string }
  | { kind: "toggle"; id: number }
  | { kind: "clear-done" };

function initial(): State {
  return { todos: [], nextId: 1 };
}

function reduce(state: State, action: Action): State {
  switch (action.kind) {
    case "add":
      return {
        ...state,
        todos: [
          ...state.todos,
          { id: state.nextId, text: action.text, done: false },
        ],
        nextId: state.nextId + 1,
      };
    case "toggle":
      return {
        ...state,
        todos: state.todos.map((t) =>
          t.id === action.id ? { ...t, done: !t.done } : t
        ),
      };
    case "clear-done":
      return { ...state, todos: state.todos.filter((t) => !t.done) };
  }
}

function selectOpenCount(state: State): number {
  return state.todos.filter((t) => !t.done).length;
}

describe("todo reducer + selector integration", () => {
  let state: State;

  beforeEach(() => {
    state = initial();
  });

  test("add then toggle flips a todo to done", () => {
    state = reduce(state, { kind: "add", text: "ship dogfood" });
    expect(state.todos).toHaveLength(1);
    expect(selectOpenCount(state)).toBe(1);

    const id = state.todos[0].id;
    state = reduce(state, { kind: "toggle", id });
    expect(state.todos[0].done).toBe(true);
    expect(selectOpenCount(state)).toBe(0);
  });

  test("clear-done removes only finished items", () => {
    state = reduce(state, { kind: "add", text: "a" });
    state = reduce(state, { kind: "add", text: "b" });
    state = reduce(state, { kind: "toggle", id: 1 });
    state = reduce(state, { kind: "clear-done" });

    expect(state.todos).toHaveLength(1);
    expect(state.todos[0].text).toBe("b");
    expect(state.todos[0].done).toBe(false);
  });
});
// CODEGEN-END
