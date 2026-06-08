interface AddTodoProps {
  onAdd: (text: string) => void;
}

export function AddTodo(props: AddTodoProps) {
  const handleSubmit = (e: Event) => {
    e.preventDefault();
    const input = document.getElementById('new-todo') as HTMLInputElement;
    const text = input.value.trim();
    if (text) {
      props.onAdd(text);
      input.value = '';
    }
  };

  return (
    <form className="add-todo" onSubmit={handleSubmit}>
      <input
        id="new-todo"
        type="text"
        placeholder="What needs to be done?"
        autoFocus={true}
      />
      <button type="submit">Add</button>
    </form>
  );
}
