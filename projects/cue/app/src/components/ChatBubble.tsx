/** @jsx createElement */
// ChatBubble — display-only sub-component of ChatTranscript.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "ChatBubble"). Tag composition:
//   `Box{ Text(role_label) + Text(model?, dim) + Markdown(content) +
//      Spinner(if pending) }`
//
// Spinner is one of the four primitive-gap issues called out in the
// mapping spec — until the renderer-contract gap closes, we emit a
// `<spinner>` intrinsic and let the renderer pick a fallback. The
// `spinner_tick` prop threads the App's frame counter through so the
// TUI fallback (rotating glyph cadence) stays deterministic.

import { createElement, type Element } from "../jsx";
import { CHAT_ROLE_LABEL } from "../protocol";
import type { ChatBubbleProps } from "../components.types";

export function ChatBubble(props: ChatBubbleProps): Element {
  const { message, spinner_tick } = props;
  return (
    <box className={`chat-bubble role-${message.role}`}>
      <text className="role-label">{CHAT_ROLE_LABEL[message.role]}</text>
      {message.model ? (
        <text className="model dim">{`(${message.model})`}</text>
      ) : null}
      <markdown className="content">{message.content}</markdown>
      {message.pending ? (
        <spinner className="pending" tick={spinner_tick} />
      ) : null}
    </box>
  );
}
