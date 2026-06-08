import { TodoItem } from './TodoItem';
import type { Todo } from '../utils/store';

interface TodoListProps {
  todos: Todo[];
  onToggle: (id: string) => void;
  onRemove: (id: string) => void;
}

export function TodoList(props: TodoListProps) {
  if (props.todos.length === 0) {
    return <p className="empty">No todos yet. Add one above!</p>;
  }

  return (
    <ul className="todo-list">
      {props.todos.map((todo: Todo) => (
        <TodoItem
          key={todo.id}
          todo={todo}
          onToggle={() => props.onToggle(todo.id)}
          onRemove={() => props.onRemove(todo.id)}
        />
      ))}
    </ul>
  );
}
