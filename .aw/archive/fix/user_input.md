Fix 4 jet bundler transformer bugs found by expanded mini-react example (#796):

1. **Spread props transform**: `<TodoItem {...todo} onToggle={f}/>` emits `createElement(TodoItem,{onToggle:f},...todo)` — spread should be merged into props object, not emitted as rest arg to createElement.

2. **Barrel re-export resolution**: `export { X } from "./X"` in barrel index.ts emits `module.exports["X"] = X` without requiring the source module — variables are undefined at runtime.

3. **Default + named export conflict**: `module.exports["default"] = V; module.exports = V` overwrites all named exports — default export should not clobber the exports object.

4. **Dynamic import() in CJS runtime**: `import("./pages/About")` is left as-is but the jet runtime uses CJS `__jet__` module system — needs async wrapper or eager resolution.