import type { Todo } from '../utils/store';

interface TodoItemProps {
  todo: Todo;
  onToggle: () => void;
  onRemove: () => void;
}

export function TodoItem(props: TodoItemProps) {
  const { todo, onToggle, onRemove } = props;
  const className = todo.done ? 'todo-item done' : 'todo-item';

  return (
    <li className={className}>
      <input
        type="checkbox"
        checked={todo.done}
        onChange={onToggle}
      />
      <span className="text">{todo.text}</span>
      <span className="date">{todo.createdAt}</span>
      <button className="remove" onClick={onRemove}>x</button>
    </li>
  );
}
