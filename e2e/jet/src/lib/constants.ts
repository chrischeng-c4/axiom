// Tests: const enum (inlined), aliased exports, computed property names,
// destructuring with defaults, as const.

export const enum LogLevel {
  Debug = 0,
  Info = 1,
  Warn = 2,
  Error = 3,
}

// Regular constant with `as const` assertion
export const FILTERS = ["all", "active", "completed"] as const;

// Aliased export
const APP_NAME = "Mini React TodoMVC";
export { APP_NAME as appName };

// Computed property key
const STATUS_KEY = "status";
export const STATUS_MAP = {
  [STATUS_KEY]: "ok",
  version: "1.0",
};

// Destructuring with defaults
export function createConfig({ theme = "light", lang = "en" } = {}) {
  return { theme, lang };
}
