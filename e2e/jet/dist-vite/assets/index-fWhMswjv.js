(function polyfill() {
  const relList = document.createElement("link").relList;
  if (relList && relList.supports && relList.supports("modulepreload")) {
    return;
  }
  for (const link of document.querySelectorAll('link[rel="modulepreload"]')) {
    processPreload(link);
  }
  new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type !== "childList") {
        continue;
      }
      for (const node of mutation.addedNodes) {
        if (node.tagName === "LINK" && node.rel === "modulepreload")
          processPreload(node);
      }
    }
  }).observe(document, { childList: true, subtree: true });
  function getFetchOpts(link) {
    const fetchOpts = {};
    if (link.integrity) fetchOpts.integrity = link.integrity;
    if (link.referrerPolicy) fetchOpts.referrerPolicy = link.referrerPolicy;
    if (link.crossOrigin === "use-credentials")
      fetchOpts.credentials = "include";
    else if (link.crossOrigin === "anonymous") fetchOpts.credentials = "omit";
    else fetchOpts.credentials = "same-origin";
    return fetchOpts;
  }
  function processPreload(link) {
    if (link.ep)
      return;
    link.ep = true;
    const fetchOpts = getFetchOpts(link);
    fetch(link.href, fetchOpts);
  }
})();
const Fragment = Symbol.for("mini-react.fragment");
function createElement(tag, props, ...children) {
  return {
    tag,
    props: props || {},
    children: children.flat().map(
      (c) => typeof c === "object" && c !== null ? c : String(c ?? "")
    )
  };
}
let currentComponent = null;
let currentHookIndex = 0;
const hookStates = /* @__PURE__ */ new Map();
const hookEffects = /* @__PURE__ */ new Map();
let rerenderQueue = /* @__PURE__ */ new Set();
let scheduledRerender = false;
function useState(initial) {
  const comp = currentComponent;
  const idx = currentHookIndex++;
  if (!hookStates.has(comp)) hookStates.set(comp, []);
  const states = hookStates.get(comp);
  if (states.length <= idx) {
    states.push(initial);
  }
  const setState = (value) => {
    const current = hookStates.get(comp)[idx];
    const next = typeof value === "function" ? value(current) : value;
    if (next !== current) {
      hookStates.get(comp)[idx] = next;
      rerenderQueue.add(comp);
      scheduleRerender();
    }
  };
  return [states[idx], setState];
}
function useEffect(callback, deps) {
  const comp = currentComponent;
  const idx = currentHookIndex++;
  if (!hookEffects.has(comp)) hookEffects.set(comp, []);
  const effects = hookEffects.get(comp);
  if (effects.length <= idx) {
    effects.push({ cb: callback, deps });
    queueMicrotask(callback);
  } else {
    const prev = effects[idx];
    const depsChanged = !deps || !prev.deps || deps.some((d, i) => d !== prev.deps[i]);
    if (depsChanged) {
      effects[idx] = { cb: callback, deps };
      queueMicrotask(callback);
    }
  }
}
let rootContainer = null;
let rootComponent = null;
function render(vnode, container) {
  if (typeof vnode === "function") {
    rootComponent = vnode;
    rootContainer = container;
    doRender();
  } else {
    container.innerHTML = "";
    container.appendChild(createDOM(vnode));
  }
}
function doRender() {
  if (!rootComponent || !rootContainer) return;
  currentComponent = rootComponent;
  currentHookIndex = 0;
  const vnode = rootComponent();
  currentComponent = null;
  rootContainer.innerHTML = "";
  rootContainer.appendChild(createDOM(vnode));
  rerenderQueue.clear();
}
function scheduleRerender() {
  if (scheduledRerender) return;
  scheduledRerender = true;
  queueMicrotask(() => {
    scheduledRerender = false;
    doRender();
  });
}
function createDOM(vnode) {
  if (typeof vnode === "string") {
    return document.createTextNode(vnode);
  }
  if (vnode.tag === Fragment) {
    const frag = document.createDocumentFragment();
    for (const child of vnode.children) {
      if (child === "false" || child === "null" || child === "undefined") continue;
      frag.appendChild(createDOM(child));
    }
    return frag;
  }
  if (typeof vnode.tag === "function") {
    currentComponent = vnode.tag;
    currentHookIndex = 0;
    const result = vnode.tag(vnode.props);
    currentComponent = null;
    return createDOM(result);
  }
  const el = document.createElement(vnode.tag);
  for (const [key, value] of Object.entries(vnode.props)) {
    if (key === "className") {
      el.setAttribute("class", value);
    } else if (key === "style" && typeof value === "object") {
      Object.assign(el.style, value);
    } else if (key.startsWith("on") && typeof value === "function") {
      const event = key.slice(2).toLowerCase();
      el.addEventListener(event, value);
    } else if (key === "checked") {
      el.checked = value;
    } else if (key === "value") {
      el.value = value;
    } else if (typeof value === "boolean") {
      if (value) el.setAttribute(key, "");
    } else {
      el.setAttribute(key, String(value));
    }
  }
  for (const child of vnode.children) {
    if (child === "false" || child === "null" || child === "undefined") continue;
    el.appendChild(createDOM(child));
  }
  return el;
}
const scriptRel = "modulepreload";
const assetsURL = function(dep) {
  return "/" + dep;
};
const seen = {};
const __vitePreload = function preload(baseModule, deps, importerUrl) {
  let promise = Promise.resolve();
  if (deps && deps.length > 0) {
    let allSettled2 = function(promises) {
      return Promise.all(
        promises.map(
          (p) => Promise.resolve(p).then(
            (value) => ({ status: "fulfilled", value }),
            (reason) => ({ status: "rejected", reason })
          )
        )
      );
    };
    document.getElementsByTagName("link");
    const cspNonceMeta = document.querySelector(
      "meta[property=csp-nonce]"
    );
    const cspNonce = (cspNonceMeta == null ? void 0 : cspNonceMeta.nonce) || (cspNonceMeta == null ? void 0 : cspNonceMeta.getAttribute("nonce"));
    promise = allSettled2(
      deps.map((dep) => {
        dep = assetsURL(dep);
        if (dep in seen) return;
        seen[dep] = true;
        const isCss = dep.endsWith(".css");
        const cssSelector = isCss ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${dep}"]${cssSelector}`)) {
          return;
        }
        const link = document.createElement("link");
        link.rel = isCss ? "stylesheet" : scriptRel;
        if (!isCss) {
          link.as = "script";
        }
        link.crossOrigin = "";
        link.href = dep;
        if (cspNonce) {
          link.setAttribute("nonce", cspNonce);
        }
        document.head.appendChild(link);
        if (isCss) {
          return new Promise((res, rej) => {
            link.addEventListener("load", res);
            link.addEventListener(
              "error",
              () => rej(new Error(`Unable to preload CSS for ${dep}`))
            );
          });
        }
      })
    );
  }
  function handlePreloadError(err) {
    const e = new Event("vite:preloadError", {
      cancelable: true
    });
    e.payload = err;
    window.dispatchEvent(e);
    if (!e.defaultPrevented) {
      throw err;
    }
  }
  return promise.then((res) => {
    for (const item of res || []) {
      if (item.status !== "rejected") continue;
      handlePreloadError(item.reason);
    }
    return baseModule().catch(handlePreloadError);
  });
};
function Header({ input, onInput, onAdd }) {
  return /* @__PURE__ */ createElement("header", null, /* @__PURE__ */ createElement("h1", null, "todos"), /* @__PURE__ */ createElement("div", { className: "input-row" }, /* @__PURE__ */ createElement(
    "input",
    {
      className: "new-todo",
      "data-testid": "new-todo",
      placeholder: "What needs to be done?",
      value: input,
      onInput: (e) => onInput(e.target.value),
      onKeydown: (e) => {
        if (e.key === "Enter") onAdd();
      }
    }
  ), /* @__PURE__ */ createElement("button", { "data-testid": "add-btn", onClick: onAdd }, "Add")));
}
function TodoItem(props) {
  const { id, text, done, onToggle, onRemove } = props;
  return /* @__PURE__ */ createElement(
    "li",
    {
      className: `todo-item ${done ? "completed" : ""}`,
      "data-testid": `todo-${id}`,
      "data-todo-id": String(id)
    },
    /* @__PURE__ */ createElement(
      "input",
      {
        type: "checkbox",
        className: "toggle",
        checked: done,
        onClick: () => onToggle(id)
      }
    ),
    /* @__PURE__ */ createElement("span", { className: "todo-text" }, text),
    done && /* @__PURE__ */ createElement("span", { className: "done-badge" }, "✓"),
    /* @__PURE__ */ createElement(
      "button",
      {
        className: "destroy",
        "data-testid": `delete-${id}`,
        onClick: () => onRemove(id)
      },
      "x"
    )
  );
}
function formatCount(count) {
  return `${count} item${count !== 1 ? "s" : ""} left`;
}
function filterTodos(todos, filter) {
  switch (filter) {
    case "active":
      return todos.filter((t) => !t.done);
    case "completed":
      return todos.filter((t) => t.done);
    default:
      return todos;
  }
}
function formatDate(date) {
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric"
  });
}
function TodoFooter({ todos, filter, onFilterChange, onClearCompleted }) {
  const remaining = todos.filter((t) => !t.done).length;
  const hasCompleted = todos.some((t) => t.done);
  return /* @__PURE__ */ createElement("footer", { className: "footer", "data-testid": "footer" }, /* @__PURE__ */ createElement("span", { className: "todo-count", "data-testid": "count" }, formatCount(remaining)), /* @__PURE__ */ createElement(Fragment, null, /* @__PURE__ */ createElement("div", { className: "filters", "data-testid": "filters" }, /* @__PURE__ */ createElement(
    "button",
    {
      className: filter === "all" ? "selected" : "",
      onClick: () => onFilterChange("all")
    },
    "All"
  ), /* @__PURE__ */ createElement(
    "button",
    {
      className: filter === "active" ? "selected" : "",
      onClick: () => onFilterChange("active")
    },
    "Active"
  ), /* @__PURE__ */ createElement(
    "button",
    {
      className: filter === "completed" ? "selected" : "",
      onClick: () => onFilterChange("completed")
    },
    "Completed"
  ))), hasCompleted ? /* @__PURE__ */ createElement(
    "button",
    {
      className: "clear-completed",
      "data-testid": "clear-completed",
      onClick: onClearCompleted
    },
    "Clear completed"
  ) : "");
}
const PI = 3.14159;
function percentage(done, total) {
  return total === 0 ? 0 : Math.round(done / total * 100);
}
function progressText(done, total) {
  const pct = percentage(done, total);
  return `${pct}% complete`;
}
const APP_NAME = "Mini React TodoMVC";
function createConfig({ theme = "light", lang = "en" } = {}) {
  return { theme, lang };
}
async function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
async function fetchItems(items) {
  await delay(0);
  const result = [];
  for (const item of items) {
    result.push(item);
  }
  return result;
}
var Priority = /* @__PURE__ */ ((Priority2) => {
  Priority2[Priority2["Low"] = 0] = "Low";
  Priority2[Priority2["Medium"] = 1] = "Medium";
  Priority2[Priority2["High"] = 2] = "High";
  return Priority2;
})(Priority || {});
const VERSION = "1.0.0";
function TodoStats({ todos }) {
  const total = todos.length;
  const done = todos.filter((t) => t.done).length;
  const pct = percentage(done, total);
  const barStyle = {
    width: `${pct}%`,
    backgroundColor: pct === 100 ? "#4caf50" : "#2196f3",
    height: "4px",
    transition: "width 0.3s"
  };
  const firstTodo = todos[0];
  const firstText = (firstTodo == null ? void 0 : firstTodo.text) ?? "No todos yet";
  const defaultPriority = Priority.Medium;
  return /* @__PURE__ */ createElement("div", { className: "todo-stats", "data-testid": "todo-stats" }, /* @__PURE__ */ createElement("div", { className: "progress-bar", "data-testid": "progress-bar" }, /* @__PURE__ */ createElement("div", { style: barStyle, "data-testid": "progress-fill" })), /* @__PURE__ */ createElement("span", { "data-testid": "progress-text" }, progressText(done, total)), /* @__PURE__ */ createElement("span", { "data-testid": "first-todo-text" }, firstText), /* @__PURE__ */ createElement("span", { "data-testid": "pi-value" }, String(PI.toFixed(2))), /* @__PURE__ */ createElement("span", { "data-testid": "priority-value" }, String(defaultPriority)));
}
function AppInfo({ todoCount }) {
  const [asyncResult, setAsyncResult] = useState("");
  const config = createConfig();
  const handleAsyncTest = () => {
    fetchItems(["a", "b", "c"]).then((items) => {
      setAsyncResult((_) => items.join(","));
    });
  };
  const statusBadge = /* @__PURE__ */ createElement("span", { className: "status-ok", "data-testid": "status-badge" }, "OK");
  return /* @__PURE__ */ createElement("div", { className: "app-info", "data-testid": "app-info" }, /* @__PURE__ */ createElement("span", { "data-testid": "app-name" }, APP_NAME), statusBadge, /* @__PURE__ */ createElement("span", { "data-testid": "config-theme" }, config.theme), /* @__PURE__ */ createElement("span", { "data-testid": "config-lang" }, config.lang), /* @__PURE__ */ createElement("span", { "data-testid": "todo-summary" }, todoCount > 0 ? `${todoCount} todos` : "empty"), /* @__PURE__ */ createElement("button", { "data-testid": "async-test-btn", onClick: handleAsyncTest }, "Test Async"), /* @__PURE__ */ createElement("span", { "data-testid": "async-result" }, asyncResult));
}
function useLocalStorage(key, initialValue) {
  const stored = localStorage.getItem(key);
  const initial = stored ? JSON.parse(stored) : initialValue;
  const [value, setValue] = useState(initial);
  const setAndStore = (v) => {
    setValue((prev) => {
      const next = typeof v === "function" ? v(prev) : v;
      localStorage.setItem(key, JSON.stringify(next));
      return next;
    });
  };
  return [value, setAndStore];
}
let nextId = 1;
function App() {
  const [todos, setTodos] = useLocalStorage("mini-react-todos", []);
  const [input, setInput] = useState("");
  const [filter, setFilter] = useState("all");
  const [showAbout, setShowAbout] = useState(false);
  const [aboutContent, setAboutContent] = useState({ data: "" });
  const [showSettings, setShowSettings] = useState(false);
  const [settingsLoaded, setSettingsLoaded] = useState(false);
  const addTodo = () => {
    const text = input.trim();
    if (!text) return;
    setTodos((prev) => [...prev, { id: nextId++, text, done: false }]);
    setInput("");
  };
  const toggleTodo = (id) => {
    setTodos(
      (prev) => prev.map((t) => t.id === id ? { ...t, done: !t.done } : t)
    );
  };
  const removeTodo = (id) => {
    setTodos((prev) => prev.filter((t) => t.id !== id));
  };
  const toggleAll = () => {
    const allDone = todos.every((t) => t.done);
    setTodos((prev) => prev.map((t) => ({ ...t, done: !allDone })));
  };
  const clearCompleted = () => {
    setTodos((prev) => prev.filter((t) => !t.done));
  };
  const loadAbout = () => {
    setShowAbout((prev) => !prev);
    if (!showAbout) {
      __vitePreload(() => import("./About-Mt8CYShk.js"), true ? [] : void 0).then(() => {
        setAboutContent((_) => ({ data: "loaded" }));
      });
    }
  };
  const loadSettings = () => {
    setShowSettings((prev) => !prev);
    if (!showSettings) {
      __vitePreload(() => import("./Settings-B1a8RmuR.js"), true ? [] : void 0).then(() => {
        setSettingsLoaded((_) => true);
      });
    }
  };
  const filtered = filterTodos(todos, filter);
  return /* @__PURE__ */ createElement("div", { className: "todoapp", "data-testid": "app" }, /* @__PURE__ */ createElement(Header, { input, onInput: setInput, onAdd: addTodo }), /* @__PURE__ */ createElement("div", { className: "version-info", "data-testid": "version" }, "v", VERSION), todos.length > 0 ? /* @__PURE__ */ createElement("section", { className: "main" }, /* @__PURE__ */ createElement(
    "button",
    {
      className: "toggle-all",
      "data-testid": "toggle-all",
      onClick: toggleAll
    },
    "Toggle All"
  ), /* @__PURE__ */ createElement("ul", { className: "todo-list", "data-testid": "todo-list" }, filtered.map((todo) => /* @__PURE__ */ createElement(
    TodoItem,
    {
      ...todo,
      onToggle: toggleTodo,
      onRemove: removeTodo
    }
  ))), /* @__PURE__ */ createElement(
    TodoFooter,
    {
      todos,
      filter,
      onFilterChange: setFilter,
      onClearCompleted: clearCompleted
    }
  ), /* @__PURE__ */ createElement(TodoStats, { todos })) : "", /* @__PURE__ */ createElement(AppInfo, { todoCount: todos.length }), /* @__PURE__ */ createElement("button", { "data-testid": "about-btn", onClick: loadAbout }, showAbout ? "Hide About" : "Show About"), showAbout && aboutContent.data ? /* @__PURE__ */ createElement("div", { className: "about-section", "data-testid": "about-section" }, /* @__PURE__ */ createElement("p", null, "About page loaded successfully."), /* @__PURE__ */ createElement("p", null, "Created: ", formatDate(/* @__PURE__ */ new Date()))) : "", /* @__PURE__ */ createElement("button", { "data-testid": "settings-btn", onClick: loadSettings }, showSettings ? "Hide Settings" : "Show Settings"), showSettings && settingsLoaded ? /* @__PURE__ */ createElement("div", { className: "settings-section", "data-testid": "settings-section" }, /* @__PURE__ */ createElement("p", null, "Settings loaded.")) : "");
}
const root = document.getElementById("root");
render(() => createElement(App, null), root);
export {
  useEffect as a,
  createElement as c,
  useState as u
};
