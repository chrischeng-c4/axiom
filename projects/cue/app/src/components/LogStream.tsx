/** @jsx createElement */
// LogStream — bounded scroll-tail of raw log lines.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "LogStream"). Tag composition:
//   `Scroll(scroll=tail){ List<Text>(lines) }`
//
// Lines are bounded upstream — the Rust `App.log` caps at 200
// (`LOG_CAPACITY`) and the TS reducer caps `log_tail` at 500
// (`LOG_TAIL_CAP`). The renderer trims to its own viewport on top
// of those caps.

import { createElement, type Element } from "../jsx";
import type { LogStreamProps } from "../components.types";

export function LogStream(props: LogStreamProps): Element {
  return (
    <scroll className="log-stream" style="scroll: tail">
      <list>
        {props.lines.map((line, idx) => (
          <text key={idx} className="log-line">
            {line}
          </text>
        ))}
      </list>
      {props.on_clear ? (
        <action_row>
          <text className="action" onClick={props.on_clear}>
            clear
          </text>
        </action_row>
      ) : null}
    </scroll>
  );
}
