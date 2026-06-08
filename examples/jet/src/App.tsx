import { Header } from './components/Header';
import { TodoList } from './components/TodoList';
import { AddTodo } from './components/AddTodo';
import { Footer } from './components/Footer';
import type { Store } from './utils/store';

interface AppProps {
  store: Store;
}

export function App(props: AppProps) {
  const { store } = props;
  return (
    <div className="app">
      <Header title="Warp Todo" count={store.todos.length} />
      <AddTodo onAdd={(text: string) => store.add(text)} />
      <TodoList
        todos={store.todos}
        onToggle={(id: string) => store.toggle(id)}
        onRemove={(id: string) => store.remove(id)}
      />
      <Footer remaining={store.remaining()} />
    </div>
  );
}
