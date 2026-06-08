/** @jsx createElement */
// CommandInput — controlled text input bound to `ChatState.input`.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "CommandInput"). Tag composition:
//   `TextInput(value, placeholder, disabled)`
//
// `value` is mirrored from `ChatState.input` — the reducer is the
// source of truth, this component is a pure projection. `disabled`
// surfaces while a turn is pending so the user can't queue a second
// submit. `on_cancel` is optional: TUI renderers bind ESC; web /
// desktop bind the Esc key; targets without an equivalent
// affordance simply omit the binding.

import { createElement, type Element } from "../jsx";
import type { CommandInputProps } from "../components.types";

export function CommandInput(props: CommandInputProps): Element {
  const { value, placeholder, disabled, on_input, on_submit, on_cancel } = props;
  return (
    <text_input
      className="command-input"
      value={value}
      placeholder={placeholder}
      disabled={disabled}
      onChange={on_input}
      onSubmit={on_submit}
      onCancel={on_cancel}
    />
  );
}
