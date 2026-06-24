---
id: projects-jet-logic-scss-sass-compilation-in-the-build-lib-css-pipeline-md
fill_sections: [logic]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: library-build-mode
    coverage: partial
    rationale: "jet SCSS/Sass compilation in build + lib CSS pipeline"
---

# jet SCSS/Sass compilation in build + lib CSS pipeline

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-scss
entry: imp
nodes:
  imp: { kind: start,    label: "resolve a style import" }
  isscss: { kind: decision, label: ".scss / .sass?" }
  css: { kind: process,  label: "plain .css: existing lightningcss path" }
  compile: { kind: process,  label: "compile SCSS->CSS (grass): nesting/vars/@use/@import" }
  pipe: { kind: process,  label: "feed compiled CSS into lightningcss pipeline" }
  done: { kind: terminal, label: "CSS in build output (lib: single style.css)" }
edges:
  - { from: imp,     to: isscss }
  - { from: isscss,  to: compile, label: "scss" }
  - { from: isscss,  to: css,     label: "css" }
  - { from: compile, to: pipe }
  - { from: css,     to: pipe }
  - { from: pipe,    to: done }
---
flowchart TD
    imp([style import]) --> isscss{.scss/.sass?}
    isscss -->|scss| compile[compile SCSS to CSS grass]
    isscss -->|css| css[existing lightningcss]
    compile --> pipe[lightningcss pipeline]
    css --> pipe
    pipe --> done([CSS in output])
```
