---
change: jet-remaining
group: production-build-pipeline
date: 2026-03-19
---

# Requirements

Implement a production-quality AOT build pipeline in cclab-jet. This includes ESM static analysis for tree shaking, code splitting via dynamic import(), shared chunks extraction, and entry point support. The pipeline must also support minification (JS, CSS, HTML), VLQ-encoded source maps with chaining, and dedicated CSS/Asset pipelines (PostCSS, Tailwind, images, fonts). Finally, provide build configuration for environment variables, path aliases, and targets. Validate the entire pipeline against real-world React apps like TodoMVC and cal.com using Playwright-based smoke tests.
