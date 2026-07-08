// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-data-fixtures-parity-grid.md#component
// CODEGEN-BEGIN
// parity-grid-v1 — hit-test correctness fixture for #2165
//
// Layout: 1280 x 720 viewport. Exercises the six hit-test challenges
// catalogued in ADR-009 §2:
//   (a) overlapping siblings with explicit z-index
//   (b) transform: translate/rotate/scale subtrees
//   (c) border-radius clip regions
//   (d) sub-pixel border alignment
//   (e) scrollable subtree with overflow content
//   (f) pointer-events: none opt-out overlapping a real target
//
// Every interactive cell carries `data-jet-semantic-id` so the parity
// gate can compare DOM-oracle hit-target vs jet wasm_hit_test target
// by semantic identity (see ADR-004).

import * as React from "react";

export default function ParityGrid(): JSX.Element {
  return (
    <div
      data-jet-semantic-id="parity-grid/root"
      style={{
        position: "relative",
        width: 1280,
        height: 720,
        background: "#f5f5f5",
        fontFamily: "system-ui, sans-serif",
        overflow: "hidden",
      }}
    >
      {/* (a) Overlapping siblings with explicit z-index */}
      <div
        data-jet-semantic-id="parity-grid/z-low"
        style={{
          position: "absolute",
          left: 40,
          top: 40,
          width: 240,
          height: 160,
          background: "#a8d0ff",
          zIndex: 1,
        }}
      />
      <div
        data-jet-semantic-id="parity-grid/z-mid"
        style={{
          position: "absolute",
          left: 120,
          top: 100,
          width: 240,
          height: 160,
          background: "#ffb38a",
          zIndex: 2,
        }}
      />
      <div
        data-jet-semantic-id="parity-grid/z-high"
        style={{
          position: "absolute",
          left: 200,
          top: 160,
          width: 240,
          height: 160,
          background: "#c2f0c2",
          zIndex: 3,
        }}
      />

      {/* (b) Transform subtree: translate + rotate + scale */}
      <div
        data-jet-semantic-id="parity-grid/transform-host"
        style={{
          position: "absolute",
          left: 520,
          top: 60,
          width: 260,
          height: 220,
          background: "#e6e6e6",
          transform: "translate(40px, 20px) rotate(12deg) scale(1.05)",
          transformOrigin: "top left",
        }}
      >
        <div
          data-jet-semantic-id="parity-grid/transform-child"
          style={{
            position: "absolute",
            left: 30,
            top: 40,
            width: 180,
            height: 120,
            background: "#d18cff",
          }}
        />
      </div>

      {/* (c) Border-radius clip region — clicks in the corner of the
            bounding box but outside the rounded silhouette should miss. */}
      <div
        data-jet-semantic-id="parity-grid/rounded"
        style={{
          position: "absolute",
          left: 880,
          top: 60,
          width: 320,
          height: 220,
          background: "#ffd24d",
          borderRadius: 110,
        }}
      />

      {/* (d) Sub-pixel border alignment — fractional offsets and a
            hairline border to provoke edge-zone disagreement. */}
      <div
        data-jet-semantic-id="parity-grid/hairline"
        style={{
          position: "absolute",
          left: 40.5,
          top: 360.5,
          width: 240.25,
          height: 140.75,
          background: "#ffffff",
          border: "0.5px solid #333",
          boxSizing: "border-box",
        }}
      />

      {/* (e) Scrollable subtree with overflow content */}
      <div
        data-jet-semantic-id="parity-grid/scroll-host"
        style={{
          position: "absolute",
          left: 320,
          top: 360,
          width: 280,
          height: 240,
          overflow: "auto",
          background: "#fafafa",
          border: "1px solid #ccc",
        }}
      >
        <div
          data-jet-semantic-id="parity-grid/scroll-inner"
          style={{ width: 600, height: 600, background: "#ddeaff" }}
        >
          <div
            data-jet-semantic-id="parity-grid/scroll-tile"
            style={{
              position: "absolute",
              left: 40,
              top: 80,
              width: 120,
              height: 80,
              background: "#88aaff",
            }}
          />
        </div>
      </div>

      {/* (f) pointer-events: none opt-out overlapping a real target */}
      <div
        data-jet-semantic-id="parity-grid/under-target"
        style={{
          position: "absolute",
          left: 660,
          top: 380,
          width: 280,
          height: 200,
          background: "#b3e0b3",
        }}
      />
      <div
        data-jet-semantic-id="parity-grid/pe-none-overlay"
        style={{
          position: "absolute",
          left: 680,
          top: 400,
          width: 240,
          height: 160,
          background: "rgba(255, 0, 0, 0.25)",
          pointerEvents: "none",
        }}
      />

      {/* Far-right anchor element for boundary-edge clicks. */}
      <div
        data-jet-semantic-id="parity-grid/anchor"
        style={{
          position: "absolute",
          left: 980,
          top: 380,
          width: 240,
          height: 200,
          background: "#cccccc",
          border: "2px solid #444",
        }}
      />
    </div>
  );
}
// CODEGEN-END
