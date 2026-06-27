// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-tests-fixtures-dom-production-build-dom-production-assets-src-main-tsx" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import "./main.css";

function App() {
  const [count, setCount] = useState(0);
  const mode = import.meta.env.MODE;
  const nodeEnv = process.env.NODE_ENV;

  return (
    <main className="shell">
      <h1>DOM Production Assets</h1>
      <p data-testid="mode">Mode: {mode}</p>
      <p data-testid="node-env">Build target: {nodeEnv}</p>
      <p className="status active">Styled status: active</p>
      <img className="brand" src="/brand.svg" alt="public asset" />
      <div className="counter">
        <span>Asset counter: {count}</span>
        <button type="button" onClick={() => setCount((value) => value + 1)}>
          Increment asset counter
        </button>
      </div>
    </main>
  );
}

createRoot(document.getElementById("root")!).render(<App />);

// </HANDWRITE>
