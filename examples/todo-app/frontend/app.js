const state = { filter: "all", todos: [] };

const elements = {
  form: document.querySelector("#todo-form"),
  title: document.querySelector("#todo-title"),
  priority: document.querySelector("#todo-priority"),
  dueDate: document.querySelector("#todo-date"),
  error: document.querySelector("#form-error"),
  list: document.querySelector("#todo-list"),
  empty: document.querySelector("#empty-state"),
  summary: document.querySelector("#task-summary"),
  template: document.querySelector("#task-template"),
  clearCompleted: document.querySelector("#clear-completed"),
  progress: document.querySelector("#daily-progress"),
  progressFill: document.querySelector("#progress-fill"),
};

function api(path, options = {}) {
  return fetch(path, {
    headers: { "Content-Type": "application/json", ...options.headers },
    ...options,
  }).then(async (response) => {
    if (response.status === 204) return null;
    const body = await response.json();
    if (!response.ok) throw new Error(body.error || "Something went wrong");
    return body;
  });
}

function filteredTodos() {
  if (state.filter === "open") return state.todos.filter((todo) => !todo.completed);
  if (state.filter === "done") return state.todos.filter((todo) => todo.completed);
  return state.todos;
}

function formatDate(value) {
  if (!value) return "No due date";
  const date = new Date(`${value}T00:00:00`);
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const tomorrow = new Date(today);
  tomorrow.setDate(today.getDate() + 1);
  if (date.getTime() === today.getTime()) return "Due today";
  if (date.getTime() === tomorrow.getTime()) return "Due tomorrow";
  return `Due ${date.toLocaleDateString(undefined, { month: "short", day: "numeric" })}`;
}

function isOverdue(todo) {
  if (!todo.due_date || todo.completed) return false;
  return new Date(`${todo.due_date}T23:59:59`).getTime() < Date.now();
}

function taskNode(todo) {
  const fragment = elements.template.content.cloneNode(true);
  const item = fragment.querySelector(".task-item");
  const complete = fragment.querySelector(".complete-button");
  const title = fragment.querySelector(".task-title");
  const priority = fragment.querySelector(".priority-badge");
  const dueDate = fragment.querySelector(".due-date");
  const remove = fragment.querySelector(".delete-button");
  item.dataset.id = todo.id;
  item.classList.toggle("is-completed", todo.completed);
  title.textContent = todo.title;
  priority.textContent = `${todo.priority} priority`;
  priority.classList.add(`priority-${todo.priority}`);
  dueDate.textContent = formatDate(todo.due_date);
  dueDate.classList.toggle("is-overdue", isOverdue(todo));
  complete.setAttribute("aria-label", todo.completed ? "Mark task incomplete" : "Mark task complete");
  complete.addEventListener("click", () => updateTodo(todo.id, { completed: !todo.completed }));
  remove.addEventListener("click", () => deleteTodo(todo.id));
  return fragment;
}

function render() {
  const visible = filteredTodos();
  const completed = state.todos.filter((todo) => todo.completed).length;
  const open = state.todos.length - completed;
  elements.list.replaceChildren(...visible.map(taskNode));
  elements.empty.hidden = visible.length !== 0;
  elements.summary.textContent = state.filter === "all"
    ? `${open} task${open === 1 ? "" : "s"} left to do`
    : `${visible.length} ${state.filter === "done" ? "completed" : "active"} task${visible.length === 1 ? "" : "s"}`;
  document.querySelector('[data-count="all"]').textContent = state.todos.length;
  document.querySelector('[data-count="open"]').textContent = open;
  document.querySelector('[data-count="done"]').textContent = completed;
  elements.progress.textContent = `${completed} of ${state.todos.length} tasks completed`;
  elements.progressFill.style.width = `${state.todos.length ? (completed / state.todos.length) * 100 : 0}%`;
  elements.clearCompleted.hidden = completed === 0;
}

async function refreshTodos() {
  const { todos } = await api("/api/todos");
  state.todos = todos;
  render();
}

async function updateTodo(id, changes) {
  try {
    await api(`/api/todos/${id}`, { method: "PATCH", body: JSON.stringify(changes) });
    await refreshTodos();
  } catch (error) {
    showError(error.message);
  }
}

async function deleteTodo(id) {
  try {
    await api(`/api/todos/${id}`, { method: "DELETE" });
    await refreshTodos();
  } catch (error) {
    showError(error.message);
  }
}

function showError(message) {
  elements.error.textContent = message;
  elements.error.hidden = false;
}

function hideError() {
  elements.error.textContent = "";
  elements.error.hidden = true;
}

elements.form.addEventListener("submit", async (event) => {
  event.preventDefault();
  hideError();
  const title = elements.title.value.trim();
  if (!title) {
    showError("Give your task a name first.");
    elements.title.focus();
    return;
  }
  const submit = elements.form.querySelector("button[type=submit]");
  submit.disabled = true;
  try {
    await api("/api/todos", {
      method: "POST",
      body: JSON.stringify({ title, priority: elements.priority.value, due_date: elements.dueDate.value || null }),
    });
    elements.form.reset();
    await refreshTodos();
    elements.title.focus();
  } catch (error) {
    showError(error.message);
  } finally {
    submit.disabled = false;
  }
});

document.querySelectorAll(".filter-button").forEach((button) => {
  button.addEventListener("click", () => {
    state.filter = button.dataset.filter;
    document.querySelectorAll(".filter-button").forEach((candidate) => candidate.classList.toggle("is-active", candidate === button));
    render();
  });
});

elements.clearCompleted.addEventListener("click", async () => {
  const completed = state.todos.filter((todo) => todo.completed);
  await Promise.all(completed.map((todo) => api(`/api/todos/${todo.id}`, { method: "DELETE" })));
  await refreshTodos();
});

const today = new Date();
document.querySelector("#date-label").textContent = today.toLocaleDateString(undefined, { weekday: "long", month: "long", day: "numeric" }).toUpperCase();
refreshTodos().catch((error) => showError(`Could not load tasks: ${error.message}`));
