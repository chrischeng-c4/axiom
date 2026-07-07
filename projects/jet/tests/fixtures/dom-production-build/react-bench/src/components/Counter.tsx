// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-tests-fixtures-dom-production-build-react-bench-src-components-counter-tsx" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import { useState, useCallback, useMemo } from "react";

export function Counter() {
  const [count, setCount] = useState(0);

  const increment = useCallback(() => setCount((c) => c + 1), []);
  const decrement = useCallback(() => setCount((c) => c - 1), []);
  const reset = useCallback(() => setCount(0), []);

  const isEven = useMemo(() => count % 2 === 0, [count]);

  return (
    <div>
      <h2>Counter: {count}</h2>
      <p>{isEven ? "Even" : "Odd"}</p>
      <button onClick={decrement}>-</button>
      <button onClick={reset}>Reset</button>
      <button onClick={increment}>+</button>
    </div>
  );
}

// </HANDWRITE>
