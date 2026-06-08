/** @jsxRuntime classic */
/** @jsx createElement */
import { createElement } from "../mini-react";

interface HeaderProps {
  input: string;
  onInput: (value: string) => void;
  onAdd: () => void;
}

export function Header({ input, onInput, onAdd }: HeaderProps) {
  return (
    <header>
      <h1>todos</h1>
      <div className="input-row">
        <input
          className="new-todo"
          data-testid="new-todo"
          placeholder="What needs to be done?"
          value={input}
          onInput={(e: any) => onInput(e.target.value)}
          onKeydown={(e: any) => {
            if (e.key === "Enter") onAdd();
          }}
        />
        <button data-testid="add-btn" onClick={onAdd}>
          Add
        </button>
      </div>
    </header>
  );
}
