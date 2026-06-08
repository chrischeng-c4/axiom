/** @jsx createElement */
// ChatTranscript — vertical scroll of ChatBubble rows.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "ChatTranscript"). Tag composition:
//   `Scroll(direction=vertical){ List<ChatBubble> }`
//
// `on_scroll_to_end` / `on_bubble_clicked` are the catalog's
// transcript-level event hooks; they're optional because the
// renderer is free to omit bindings the target can't surface
// (the TUI fallback has no notion of "click on bubble #3").

import { createElement, type Element } from "../jsx";
import { ChatBubble } from "./ChatBubble";
import type { ChatTranscriptProps } from "../components.types";

export function ChatTranscript(props: ChatTranscriptProps): Element {
  const { messages, spinner_tick, on_scroll_to_end, on_bubble_clicked } = props;
  return (
    <scroll
      className="chat-transcript"
      style="direction: vertical"
      onScrollEnd={on_scroll_to_end}
    >
      <list>
        {messages.map((message, idx) => (
          <box
            key={idx}
            className="chat-transcript-row"
            onClick={
              on_bubble_clicked ? () => on_bubble_clicked(idx) : undefined
            }
          >
            <ChatBubble message={message} spinner_tick={spinner_tick} />
          </box>
        ))}
      </list>
    </scroll>
  );
}
