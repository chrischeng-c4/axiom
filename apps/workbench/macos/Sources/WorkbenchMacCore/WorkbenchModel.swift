// HANDWRITE-BEGIN gap="missing-generator:logic:fa1274c4" tracker="pending-tracker" reason="Own four idle default tabs, added Shell tabs, selected folder, explicit launch, per-tab output and lifecycle, polling, and command routing."
import Combine
import Foundation

public enum TerminalLifecycle: Equatable, Sendable {
    case idle
    case starting
    case running
    case exited(UInt32?)
    case failed(String)

    public var label: String {
        switch self {
        case .idle: "Idle"
        case .starting: "Starting"
        case .running: "Running"
        case let .exited(code): code.map { "Exited \($0)" } ?? "Exited"
        case .failed: "Needs attention"
        }
    }

    public var isRunning: Bool {
        self == .running || self == .starting
    }

    public var isFailed: Bool {
        if case .failed = self { return true }
        return false
    }
}

public struct TerminalTab: Identifiable, Equatable, Sendable {
    public let id: String
    public let profile: TerminalProfile
    public var title: String
    public var lifecycle: TerminalLifecycle
    public var activeCwd: String?
    public var cwdSource: String
    public var processId: UInt32?
    public var output: Data
    public var lastSequence: UInt64

    public init(id: String, profile: TerminalProfile, title: String) {
        self.id = id
        self.profile = profile
        self.title = title
        lifecycle = .idle
        activeCwd = nil
        cwdSource = "Launch folder"
        processId = nil
        output = Data()
        lastSequence = 0
    }

    public var accessibilityLabel: String {
        "\(title), \(lifecycle.label)"
    }
}

/// The project-scoped values the native workspace renders together.
///
/// Keeping these values in one published value prevents the sidebar, launch
/// root, and Files column from observing different projects during selection.
public struct SelectedProjectWorkspace: Equatable, Sendable {
    public let project: RegisteredProject
    public let launchFolder: URL
    public let fileListing: ProjectFileListingState

    public init(project: RegisteredProject, fileListing: ProjectFileListingState) {
        self.project = project
        launchFolder = URL(fileURLWithPath: project.rootPath, isDirectory: true)
        self.fileListing = fileListing
    }
}

/// Native presentation state; every process action is delegated to the Rust core.
///
/// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#logic
@MainActor
public final class WorkbenchModel: ObservableObject {
    public static let defaultTabs = [
        TerminalTab(id: "claude", profile: .claude, title: "Claude Code"),
        TerminalTab(id: "codex", profile: .codex, title: "Codex"),
        TerminalTab(id: "agy", profile: .agy, title: "AGY"),
        TerminalTab(id: "shell", profile: .shell, title: "Shell"),
    ]

    @Published public private(set) var tabs: [TerminalTab]
    @Published public private(set) var activeTabId: String
    @Published public private(set) var projects: [RegisteredProject]
    @Published public private(set) var selectedWorkspace: SelectedProjectWorkspace?
    @Published public private(set) var statusMessage = "Add a project, then explicitly start a terminal."

    private let client: any CoreClientProtocol
    private let projectStore: ProjectStore
    private let fileListing: ProjectFileListing
    private var pollTask: Task<Void, Never>?

    public init(
        client: any CoreClientProtocol = RustCoreClient(),
        projectStore: ProjectStore = ProjectStore(),
        fileListing: ProjectFileListing = ProjectFileListing()
    ) {
        self.client = client
        self.projectStore = projectStore
        self.fileListing = fileListing
        tabs = Self.defaultTabs
        activeTabId = Self.defaultTabs[0].id
        selectedWorkspace = nil
        let loaded = projectStore.load()
        projects = loaded
        if let first = loaded.first {
            selectProject(first.id)
            statusMessage = "Restored project \(first.displayName)."
        }
    }

    public var activeTab: TerminalTab? {
        tabs.first { $0.id == activeTabId }
    }

    public var selectedProjectId: String? {
        selectedWorkspace?.project.id
    }

    public var selectedFolder: URL? {
        selectedWorkspace?.launchFolder
    }

    public var projectFileListing: ProjectFileListingState {
        selectedWorkspace?.fileListing ?? .noProject
    }

    public func registerProject(_ url: URL) {
        let project = projectStore.register(url: url)
        projects = projectStore.load()
        selectProject(project.id)
        statusMessage = "Added \(project.displayName). New terminals will start in \(project.rootPath)."
    }

    public func selectProject(_ id: String) {
        guard let project = projects.first(where: { $0.id == id }) else { return }
        let launchFolder = URL(fileURLWithPath: project.rootPath, isDirectory: true)
        selectedWorkspace = SelectedProjectWorkspace(
            project: project,
            fileListing: fileListing.load(root: launchFolder)
        )
    }

    public func removeProject(_ id: String) {
        guard let project = projects.first(where: { $0.id == id }) else { return }
        projectStore.remove(id: id)
        projects = projectStore.load()
        if selectedProjectId == id {
            selectedWorkspace = nil
            if let next = projects.first {
                selectProject(next.id)
            }
        }
        statusMessage = "Removed \(project.displayName) from Workbench. Its files were not changed."
    }

    public func selectTab(_ id: String) {
        guard tabs.contains(where: { $0.id == id }) else { return }
        activeTabId = id
    }

    public func selectDefaultTab(at index: Int) {
        guard tabs.indices.contains(index) else { return }
        activeTabId = tabs[index].id
    }

    public func addShellTab() {
        let number = tabs.filter { $0.profile == .shell }.count + 1
        let id = "shell-\(number)"
        tabs.append(TerminalTab(id: id, profile: .shell, title: "Shell \(number)"))
        activeTabId = id
        statusMessage = "Shell \(number) is idle. Press Start when you are ready."
    }

    public func closeTab(_ id: String) async {
        guard let index = tabIndex(id) else { return }
        let tab = tabs[index]
        if tab.lifecycle.isRunning {
            do {
                _ = try await client.send(
                    method: .terminate,
                    params: CoreParams(tabId: id)
                )
            } catch {
                // Ignore termination errors during close
            }
        }
        tabs.remove(at: index)
        if activeTabId == id {
            if !tabs.isEmpty {
                let nextIndex = min(index, tabs.count - 1)
                activeTabId = tabs[nextIndex].id
            } else {
                tabs = Self.defaultTabs
                activeTabId = Self.defaultTabs[0].id
            }
        }
        statusMessage = "Closed \(tab.title)."
    }

    public func startActiveTab() async {
        guard let folder = selectedFolder else {
            statusMessage = "Choose a launch folder before starting \(activeTab?.title ?? "a terminal")."
            return
        }
        guard let index = tabIndex(activeTabId), !tabs[index].lifecycle.isRunning else { return }
        tabs[index].lifecycle = .starting
        let tab = tabs[index]
        do {
            let response = try await client.send(
                method: .launch,
                params: CoreParams(
                    tabId: tab.id,
                    profile: tab.profile,
                    cwd: folder.path,
                    rows: 28,
                    cols: 100
                )
            )
            try apply(response, expectedTabId: tab.id)
            statusMessage = "\(tab.title) started in \(folder.path)."
        } catch {
            markFailed(tab.id, error: error)
        }
    }

    public func sendInput(tabId: String, data: Data) async {
        guard !data.isEmpty, tabs.first(where: { $0.id == tabId })?.lifecycle == .running else {
            return
        }
        do {
            let response = try await client.send(
                method: .input,
                params: CoreParams(tabId: tabId, dataBase64: data.base64EncodedString())
            )
            try apply(response, expectedTabId: tabId)
        } catch {
            markFailed(tabId, error: error)
        }
    }

    public func resize(tabId: String, rows: UInt16, cols: UInt16) async {
        guard rows > 0, cols > 0,
              tabs.first(where: { $0.id == tabId })?.lifecycle == .running
        else { return }
        do {
            let response = try await client.send(
                method: .resize,
                params: CoreParams(tabId: tabId, rows: rows, cols: cols)
            )
            try apply(response, expectedTabId: tabId)
        } catch {
            markFailed(tabId, error: error)
        }
    }

    public func interruptActiveTab() async {
        await runActiveLifecycleMethod(.interrupt)
    }

    public func stopActiveTab() async {
        await runActiveLifecycleMethod(.terminate)
    }

    public func beginPolling() {
        guard pollTask == nil else { return }
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(for: .milliseconds(100))
                await self?.pollRunningTabs()
            }
        }
    }

    public func endPolling() async {
        pollTask?.cancel()
        pollTask = nil
        await client.shutdown()
    }

    public func pollRunningTabs() async {
        let runningIds = tabs.filter { $0.lifecycle == .running }.map(\.id)
        for tabId in runningIds {
            do {
                let response = try await client.send(
                    method: .poll,
                    params: CoreParams(tabId: tabId)
                )
                try apply(response, expectedTabId: tabId)
            } catch {
                markFailed(tabId, error: error)
            }
        }
    }

    private func runActiveLifecycleMethod(_ method: CoreMethod) async {
        guard let tab = activeTab, tab.lifecycle == .running else { return }
        do {
            let response = try await client.send(
                method: method,
                params: CoreParams(tabId: tab.id)
            )
            try apply(response, expectedTabId: tab.id)
            statusMessage = method == .terminate ? "\(tab.title) stopped." : "Interrupted \(tab.title)."
        } catch {
            markFailed(tab.id, error: error)
        }
    }

    private func apply(_ response: CoreResponse, expectedTabId: String) throws {
        guard response.protocolVersion == workbenchCoreProtocolVersion else {
            throw CoreClientError.protocolMismatch(response.protocolVersion)
        }
        guard let frame = response.result?.frame else {
            throw CoreClientError.transport("workbench-core returned no terminal frame")
        }
        guard frame.snapshot.tabId == expectedTabId else {
            throw CoreClientError.transport(
                "workbench-core routed tab \(frame.snapshot.tabId) to \(expectedTabId)"
            )
        }
        guard let index = tabIndex(expectedTabId) else { return }
        if frame.sequence > tabs[index].lastSequence,
           let bytes = Data(base64Encoded: frame.outputBase64)
        {
            tabs[index].output.append(bytes)
            if tabs[index].output.count > 2 * 1024 * 1024 {
                tabs[index].output.removeFirst(tabs[index].output.count - 2 * 1024 * 1024)
            }
            tabs[index].lastSequence = frame.sequence
        }
        tabs[index].activeCwd = frame.snapshot.activeCwd
        tabs[index].cwdSource = frame.snapshot.cwdSource
        tabs[index].processId = frame.snapshot.processId
        tabs[index].lifecycle = frame.snapshot.running ? .running : .exited(frame.snapshot.exitCode)
    }

    private func markFailed(_ tabId: String, error: Error) {
        guard let index = tabIndex(tabId) else { return }
        let message = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        tabs[index].lifecycle = .failed(message)
        statusMessage = message
        WorkbenchDiagnosticLog.write("terminal.failed", details: [
            "tab": tabId,
            "profile": tabs[index].profile.rawValue,
            "error": message,
        ])
    }

    private func tabIndex(_ id: String) -> Int? {
        tabs.firstIndex { $0.id == id }
    }
}
// HANDWRITE-END
