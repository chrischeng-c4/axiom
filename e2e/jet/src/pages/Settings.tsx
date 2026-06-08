/** @jsxRuntime classic */
/** @jsx createElement */
import { createElement, useState, useEffect } from "../mini-react";

// Second dynamic import target — tests multiple lazy loads.
// Also tests useEffect + export default function declaration.
export default function Settings() {
  const [theme, setTheme] = useState("light");
  const [mounted, setMounted] = useState(false);

  // useEffect: set mounted flag after render
  useEffect(() => {
    setMounted((_: boolean) => true);
  }, []);

  return (
    <div className="settings-page" data-testid="settings-page">
      <h3>Settings</h3>
      <div data-testid="settings-mounted">{mounted ? "ready" : "loading"}</div>
      <div data-testid="settings-theme">
        <span>Theme: {theme}</span>
        <button
          data-testid="toggle-theme"
          onClick={() =>
            setTheme((t: string) => (t === "light" ? "dark" : "light"))
          }
        >
          Toggle
        </button>
      </div>
    </div>
  );
}
