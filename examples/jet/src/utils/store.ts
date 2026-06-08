import { generateId } from './id';
import { formatTime } from './time';

export interface Todo {
  id: string;
  text: string;
  done: boolean;
  createdAt: string;
}

export interface Store {
  todos: Todo[];
  add: (text: string) => void;
  toggle: (id: string) => void;
  remove: (id: string) => void;
  remaining: () => number;
}

export function createStore(): Store {
  const todos: Todo[] = [];

  return {
    todos,

    add(text: string) {
      todos.push({
        id: generateId(),
        text,
        done: false,
        createdAt: formatTime(new Date()),
      });
    },

    toggle(id: string) {
      const todo = todos.find((t: Todo) => t.id === id);
      if (todo) {
        todo.done = !todo.done;
      }
    },

    remove(id: string) {
      const idx = todos.findIndex((t: Todo) => t.id === id);
      if (idx !== -1) {
        todos.splice(idx, 1);
      }
    },

    remaining() {
      return todos.filter((t: Todo) => !t.done).length;
    },
  };
}
