/** @jsxRuntime classic */
/** @jsx createElement */
import { createElement } from "../mini-react";

// Lazy-loaded About page — dynamic import() target (R6).
export default function About() {
  return (
    <div className="about-page" data-testid="about-page">
      <h2>About Mini React TodoMVC</h2>
      <p>A minimal React-compatible implementation for testing the jet bundler.</p>
      <p>Built with mini-react — no virtual DOM, direct DOM manipulation.</p>
    </div>
  );
}
