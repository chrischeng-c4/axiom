// HANDWRITE-BEGIN gap="missing-generator:logic:f6671efc" tracker="pending-tracker" reason="Bootstrap the macOS-only SwiftUI application and native commands."
import AppKit
import SwiftUI
#if canImport(WorkbenchMacCore)
import WorkbenchMacCore
#endif

/// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#logic
@main
struct WorkbenchMacApp: App {
    @StateObject private var model: WorkbenchModel
    private let localRuntime: LocalRuntimeServer

    init() {
        let runtimeProfile = WorkbenchRuntimeProfile.from()
        WorkbenchDiagnosticLog.configure(profile: runtimeProfile)
        let model = WorkbenchModel(
            projectStore: ProjectStore(storageDirectory: runtimeProfile.projectsRoot())
        )
        let localRuntime = LocalRuntimeServer(runtimeDirectory: runtimeProfile.runtimeRoot())
        do {
            try localRuntime.start()
        } catch LocalRuntimeError.alreadyRunning {
            WorkbenchDiagnosticLog.write("runtime.already_running")
            DispatchQueue.main.async { NSApp.terminate(nil) }
        } catch {
            WorkbenchDiagnosticLog.write("runtime.start_failed", details: ["error": error.localizedDescription])
        }
        WorkbenchDiagnosticLog.write("app.started", details: [
            "executable": Bundle.main.executableURL?.path ?? "unknown",
            "profile": runtimeProfile.rawValue,
        ])
        if let fixtureFolder = ProcessInfo.processInfo.environment["WORKBENCH_UI_TEST_FOLDER"],
           !fixtureFolder.isEmpty
        {
            model.registerProject(URL(fileURLWithPath: fixtureFolder, isDirectory: true))
        }
        if let initialTab = ProcessInfo.processInfo.environment["WORKBENCH_UI_TEST_ACTIVE_TAB"],
           !initialTab.isEmpty
        {
            model.selectTab(initialTab)
        }
        _model = StateObject(wrappedValue: model)
        self.localRuntime = localRuntime
    }

    var body: some Scene {
        WindowGroup("Workbench") {
            WorkbenchView(model: model)
                .frame(minWidth: 760, minHeight: 520)
                .background(AppOwnedWindowChrome())
                .ignoresSafeArea(.container, edges: .top)
                .task { model.beginPolling() }
                .onDisappear {
                    Task { await model.endPolling() }
                    localRuntime.stop()
                }
        }
        .defaultSize(width: 1280, height: 820)
        .windowStyle(.hiddenTitleBar)
        .commands {
            CommandMenu("Terminal") {
                Button("New Shell Tab") { model.addShellTab() }
                    .keyboardShortcut("t", modifiers: [.command])
                Divider()
                Button("Start Active Terminal") {
                    Task { await model.startActiveTab() }
                }
                .keyboardShortcut(.return, modifiers: [.command])
                Button("Interrupt Active Terminal") {
                    Task { await model.interruptActiveTab() }
                }
                .keyboardShortcut("c", modifiers: [.command, .shift])
                Button("Stop Active Terminal") {
                    Task { await model.stopActiveTab() }
                }
                Divider()
                ForEach(Array(model.tabs.prefix(9).enumerated()), id: \.element.id) { index, tab in
                    Button("Select \(tab.title)") { model.selectDefaultTab(at: index) }
                        .keyboardShortcut(
                            KeyEquivalent(Character(String(index + 1))),
                            modifiers: [.command]
                        )
                }
            }
        }
    }
}

/// Makes the window's titlebar a visual part of the app without moving any
/// controls into native titlebar or toolbar territory. Native chrome is hidden
/// by macOS in fullscreen; this represented view only makes app content reach
/// the top edge so terminal tabs stay visible in every window mode.
private struct AppOwnedWindowChrome: NSViewRepresentable {
    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> NSView {
        let view = NSView(frame: .zero)
        DispatchQueue.main.async {
            context.coordinator.configure(view.window)
        }
        return view
    }

    func updateNSView(_ view: NSView, context: Context) {
        context.coordinator.configure(view.window)
    }

    final class Coordinator {
        private weak var configuredWindow: NSWindow?

        func configure(_ window: NSWindow?) {
            guard let window, configuredWindow !== window else { return }
            window.titleVisibility = .hidden
            window.titlebarAppearsTransparent = true
            window.styleMask.insert(.fullSizeContentView)
            window.isMovableByWindowBackground = true
            configuredWindow = window
        }
    }
}
// HANDWRITE-END
