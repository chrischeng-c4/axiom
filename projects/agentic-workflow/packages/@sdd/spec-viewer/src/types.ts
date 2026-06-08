// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/spec-viewer/src/types.md#source
// CODEGEN-BEGIN
export interface SpecViewerProps {
  /** Markdown content to render */
  content: string
  /** Optional CSS class name */
  className?: string
  /** Enable dark mode (auto-detected from prefers-color-scheme by default) */
  darkMode?: boolean
}

export interface MermaidDiagramProps {
  /** Mermaid diagram source code */
  content: string
  /** Optional CSS class name */
  className?: string
}

export interface CodeBlockProps {
  /** Code content */
  code: string
  /** Language identifier for syntax class */
  language?: string
  /** Optional CSS class name */
  className?: string
}


// CODEGEN-END
