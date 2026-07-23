// HANDWRITE-BEGIN gap="missing-generator:logic:3ebfd6b8" tracker="pending-tracker" reason="Embed SwiftTerm TerminalView through NSViewRepresentable and route raw input and resize only to the represented tab."
import AppKit
import SwiftTerm
import SwiftUI

/// SwiftTerm renders terminal bytes; Rust remains the only PTY/process owner.
///
/// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#logic
struct TerminalSurface: NSViewRepresentable {
    let tabId: String
    let output: Data
    let acceptsInput: Bool
    let onInput: (Data) -> Void
    let onResize: (UInt16, UInt16) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(
            tabId: tabId,
            acceptsInput: acceptsInput,
            onInput: onInput,
            onResize: onResize
        )
    }

    func makeNSView(context: Context) -> TerminalView {
        let terminal = TerminalView(frame: .zero)
        terminal.terminalDelegate = context.coordinator
        terminal.setAccessibilityElement(true)
        terminal.setAccessibilityRole(.textArea)
        terminal.setAccessibilityIdentifier("terminal.surface.\(tabId)")
        terminal.setAccessibilityLabel("\(tabId) native terminal")
        terminal.changeScrollback(10_000)
        return terminal
    }

    /// Keep the rendered terminal dense and quiet.  The terminal owns this
    /// appearance; SwiftUI should only arrange the surrounding workspace.
    private func configureTerminalAppearance(_ terminal: TerminalView) {
        terminal.font = NSFont(name: "Menlo-Regular", size: 13)
            ?? .monospacedSystemFont(ofSize: 13, weight: .regular)
        terminal.lineSpacing = 1.08
        terminal.fontSmoothing = false
        terminal.nativeBackgroundColor = NSColor(
            calibratedRed: 0.118,
            green: 0.118,
            blue: 0.118,
            alpha: 1
        )
        terminal.nativeForegroundColor = NSColor(
            calibratedRed: 0.898,
            green: 0.898,
            blue: 0.898,
            alpha: 1
        )
        terminal.installColors(workbenchAnsiPalette)
        terminal.selectedTextBackgroundColor = NSColor(
            calibratedRed: 0.255,
            green: 0.423,
            blue: 0.620,
            alpha: 0.72
        )
        terminal.caretColor = NSColor(
            calibratedRed: 0.925,
            green: 0.925,
            blue: 0.925,
            alpha: 1
        )
        terminal.caretViewTracksFocus = true
        terminal.optionAsMetaKey = true
    }

    /// The first sixteen ANSI entries are the theme contract. SwiftTerm
    /// deterministically derives the remaining 240 xterm entries from them,
    /// while SGR truecolor continues to render its supplied RGB value.
    private var workbenchAnsiPalette: [SwiftTerm.Color] {
        [
            rgb(0x15, 0x15, 0x15), rgb(0xC0, 0x1C, 0x28),
            rgb(0x26, 0xA2, 0x69), rgb(0xA2, 0x73, 0x4C),
            rgb(0x12, 0x48, 0x8B), rgb(0xA3, 0x47, 0xBA),
            rgb(0x2A, 0xA1, 0xB3), rgb(0xD0, 0xCF, 0xCC),
            rgb(0x4E, 0x4E, 0x4E), rgb(0xF6, 0x61, 0x51),
            rgb(0x33, 0xD1, 0x7A), rgb(0xE9, 0xAD, 0x0C),
            rgb(0x2A, 0x7B, 0xDE), rgb(0xC0, 0x61, 0xCB),
            rgb(0x33, 0xC7, 0xDE), rgb(0xFF, 0xFF, 0xFF),
        ]
    }

    private func rgb(_ red: UInt8, _ green: UInt8, _ blue: UInt8) -> SwiftTerm.Color {
        SwiftTerm.Color(
            red: UInt16(red) * 257,
            green: UInt16(green) * 257,
            blue: UInt16(blue) * 257
        )
    }

    func updateNSView(_ terminal: TerminalView, context: Context) {
        context.coordinator.tabId = tabId
        context.coordinator.acceptsInput = acceptsInput
        context.coordinator.onInput = onInput
        context.coordinator.onResize = onResize
        context.coordinator.configureAppearanceIfNeeded(terminal, configure: configureTerminalAppearance)
        context.coordinator.feedNewBytes(output, into: terminal)
        terminal.setAccessibilityIdentifier("terminal.surface.\(tabId)")
        terminal.setAccessibilityLabel("\(tabId) native terminal")
        terminal.setAccessibilityValue(
            String(decoding: output.suffix(65_536), as: UTF8.self)
        )
        if acceptsInput, terminal.window?.firstResponder !== terminal {
            DispatchQueue.main.async {
                terminal.window?.makeFirstResponder(terminal)
            }
        }
        // SwiftUI may give the represented NSView its final bounds after the
        // terminal session has already been launched with its conservative
        // fallback size. Report SwiftTerm's calculated grid on the next run
        // loop so the Rust PTY immediately matches the visible surface.
        DispatchQueue.main.async {
            context.coordinator.reportCurrentGrid(of: terminal)
        }
    }

    final class Coordinator: NSObject, TerminalViewDelegate {
        var tabId: String
        var acceptsInput: Bool
        var onInput: (Data) -> Void
        var onResize: (UInt16, UInt16) -> Void
        private var fedByteCount = 0
        private var didConfigureAppearance = false
        private var lastReportedGrid: (rows: UInt16, cols: UInt16)?

        init(
            tabId: String,
            acceptsInput: Bool,
            onInput: @escaping (Data) -> Void,
            onResize: @escaping (UInt16, UInt16) -> Void
        ) {
            self.tabId = tabId
            self.acceptsInput = acceptsInput
            self.onInput = onInput
            self.onResize = onResize
        }

        func configureAppearanceIfNeeded(
            _ terminal: TerminalView,
            configure: (TerminalView) -> Void
        ) {
            guard !didConfigureAppearance else { return }
            configure(terminal)
            didConfigureAppearance = true
        }

        func feedNewBytes(_ output: Data, into terminal: TerminalView) {
            if output.count < fedByteCount {
                fedByteCount = 0
            }
            guard output.count > fedByteCount else { return }
            let bytes = Array(output[fedByteCount...])
            terminal.feed(byteArray: bytes[...])
            fedByteCount = output.count
        }

        func reportCurrentGrid(of terminal: TerminalView) {
            guard acceptsInput else { return }
            let engine = terminal.getTerminal()
            let grid = (
                rows: UInt16(clamping: engine.rows),
                cols: UInt16(clamping: engine.cols)
            )
            guard grid.rows > 0, grid.cols > 0 else { return }
            if let lastReportedGrid,
               lastReportedGrid.rows == grid.rows,
               lastReportedGrid.cols == grid.cols
            {
                return
            }
            lastReportedGrid = grid
            onResize(grid.rows, grid.cols)
        }

        func send(source _: TerminalView, data: ArraySlice<UInt8>) {
            guard acceptsInput else { return }
            onInput(Data(data))
        }

        func sizeChanged(source _: TerminalView, newCols: Int, newRows: Int) {
            guard acceptsInput, newCols > 0, newRows > 0 else { return }
            onResize(
                UInt16(clamping: newRows),
                UInt16(clamping: newCols)
            )
        }

        func setTerminalTitle(source _: TerminalView, title _: String) {}
        func hostCurrentDirectoryUpdate(source _: TerminalView, directory _: String?) {}
        func scrolled(source _: TerminalView, position _: Double) {}
        func clipboardCopy(source _: TerminalView, content _: Data) {}
        func rangeChanged(source _: TerminalView, startY _: Int, endY _: Int) {}
    }
}
// HANDWRITE-END
