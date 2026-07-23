// HANDWRITE-BEGIN gap="missing-generator:logic:f6671efc" tracker="pending-tracker" reason="Bootstrap the macOS-only SwiftUI application and native commands."
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
        let model = WorkbenchModel()
        let localRuntime = LocalRuntimeServer()
        do {
            try localRuntime.start()
        } catch {
            WorkbenchDiagnosticLog.write("runtime.start_failed", details: ["error": error.localizedDescription])
        }
        WorkbenchDiagnosticLog.write("app.started", details: [
            "executable": Bundle.main.executableURL?.path ?? "unknown",
        ])
        if let fixtureFolder = ProcessInfo.processInfo.environment["WORKBENCH_UI_TEST_FOLDER"],
           !fixtureFolder.isEmpty
        {
            model.registerProject(URL(fileURLWithPath: fixtureFolder, isDirectory: true))
        }
        _model = StateObject(wrappedValue: model)
        self.localRuntime = localRuntime
    }

    var body: some Scene {
        WindowGroup("Workbench") {
            WorkbenchView(model: model)
                .frame(minWidth: 760, minHeight: 520)
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
// HANDWRITE-END
