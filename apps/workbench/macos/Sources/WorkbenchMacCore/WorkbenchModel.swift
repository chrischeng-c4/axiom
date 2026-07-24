// HANDWRITE-BEGIN gap="missing-generator:logic:fa1274c4" tracker="pending-tracker" reason="Own project-scoped terminal panes, explicit profile launch, per-session output and lifecycle, polling, and command routing."
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

/// A terminal session belongs to one registered project. `id` is qualified for
/// SwiftUI so same-named sessions from separate projects never share a renderer.
public struct ProjectTerminalTab: Identifiable, Equatable, Sendable {
    public let projectId: String
    public let tab: TerminalTab

    public var id: String { "\(projectId)::\(tab.id)" }
}

public struct TerminalPane: Identifiable, Equatable, Sendable {
    public let id: String
    public var tabId: String?

    public init(id: String = UUID().uuidString, tabId: String? = nil) {
        self.id = id
        self.tabId = tabId
    }
}

private struct ProjectTerminalWorkspace: Equatable, Sendable {
    var tabs: [TerminalTab]
    var activeTabId: String
    var panes: [TerminalPane]
    var activePaneId: String

    init(tabs: [TerminalTab], activeTabId: String, panes: [TerminalPane], activePaneId: String) {
        self.tabs = tabs
        self.activeTabId = activeTabId
        self.panes = panes
        self.activePaneId = activePaneId
    }
}

/// Native presentation state; every process action is delegated to the Rust core.
///
/// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#logic
@MainActor
public final class WorkbenchModel: ObservableObject {
    public static let defaultTabs: [TerminalTab] = []

    @Published public private(set) var tabs: [TerminalTab]
    @Published public private(set) var activeTabId: String
    @Published public private(set) var panes: [TerminalPane]
    @Published public private(set) var activePaneId: String
    @Published public private(set) var projects: [RegisteredProject]
    @Published public private(set) var selectedWorkspace: SelectedProjectWorkspace?
    @Published public private(set) var statusMessage = "Add a project, then explicitly start a terminal."

    private let client: any CoreClientProtocol
    private let projectStore: ProjectStore
    private let fileListing: ProjectFileListing
    private var pollTask: Task<Void, Never>?
    private var projectTerminalWorkspaces: [String: ProjectTerminalWorkspace] = [:]

    public init(
        client: any CoreClientProtocol = RustCoreClient(),
        projectStore: ProjectStore = ProjectStore(),
        fileListing: ProjectFileListing = ProjectFileListing()
    ) {
        self.client = client
        self.projectStore = projectStore
        self.fileListing = fileListing
        tabs = Self.defaultTabs
        activeTabId = ""
        let firstPane = TerminalPane()
        panes = [firstPane]
        activePaneId = firstPane.id
        selectedWorkspace = nil
        let loaded = projectStore.load()
        projects = loaded
        if let first = loaded.first {
            selectProject(first.id)
            statusMessage = "Restored project \(first.displayName)."
        }
    }

    public var activeTab: TerminalTab? {
        tabs.first { $0.id == panes.first(where: { $0.id == activePaneId })?.tabId }
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

    /// Every running renderer stays mounted while its project is registered.
    /// Only the selected project is interactive and visible, but retaining its
    /// AppKit view prevents transcript replay when a user returns to a tab.
    public var mountedTerminalTabs: [ProjectTerminalTab] {
        projectTerminalWorkspaces.flatMap { projectId, workspace in
            let visibleTabs = projectId == selectedProjectId ? tabs : workspace.tabs
            return visibleTabs.map { ProjectTerminalTab(projectId: projectId, tab: $0) }
        }
    }

    public func registerProject(_ url: URL) {
        let hadSelectedProject = selectedProjectId != nil
        let project = projectStore.register(url: url)
        projects = projectStore.load()
        if !hadSelectedProject {
            projectTerminalWorkspaces[project.id] = ProjectTerminalWorkspace(
                tabs: tabs,
                activeTabId: activeTabId, panes: panes, activePaneId: activePaneId
            )
        }
        selectProject(project.id)
        statusMessage = "Added \(project.displayName). New terminals will start in \(project.rootPath)."
    }

    public func selectProject(_ id: String) {
        guard let project = projects.first(where: { $0.id == id }) else { return }
        if selectedProjectId == id { return }
        saveSelectedProjectTerminalWorkspace()
        let firstPane = TerminalPane()
        let terminalWorkspace = projectTerminalWorkspaces[id] ?? ProjectTerminalWorkspace(
            tabs: [], activeTabId: "", panes: [firstPane], activePaneId: firstPane.id
        )
        projectTerminalWorkspaces[id] = terminalWorkspace
        tabs = terminalWorkspace.tabs
        activeTabId = terminalWorkspace.activeTabId
        panes = terminalWorkspace.panes
        activePaneId = terminalWorkspace.activePaneId
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
            saveSelectedProjectTerminalWorkspace()
            selectedWorkspace = nil
            if let next = projects.first {
                selectProject(next.id)
            }
        }
        statusMessage = "Removed \(project.displayName) from Workbench. Its files were not changed."
    }

    public func selectTab(_ id: String) {
        guard let pane = panes.first(where: { $0.tabId == id }) else { return }
        selectPane(pane.id)
    }

    public func selectPane(_ id: String) {
        guard let pane = panes.first(where: { $0.id == id }) else { return }
        activePaneId = pane.id
        activeTabId = pane.tabId ?? ""
    }

    public func selectDefaultTab(at index: Int) {
        guard tabs.indices.contains(index) else { return }
        selectTab(tabs[index].id)
    }

    public func addShellTab() {
        addTerminal(profile: .shell)
    }

    public func addTerminal(profile: TerminalProfile) {
        guard let paneIndex = panes.firstIndex(where: { $0.id == activePaneId }) else { return }
        if panes[paneIndex].tabId != nil, panes.count < 2 {
            let pane = TerminalPane()
            panes.append(pane)
            activePaneId = pane.id
        }
        guard let targetIndex = panes.firstIndex(where: { $0.id == activePaneId }), panes[targetIndex].tabId == nil else {
            statusMessage = "Close a pane before adding another terminal."
            return
        }
        let number = tabs.filter { $0.profile == profile }.count + 1
        let id = number == 1 ? profile.rawValue : "\(profile.rawValue)-\(number)"
        let title = number == 1 ? profile.label : "\(profile.label) \(number)"
        tabs.append(TerminalTab(id: id, profile: profile, title: title))
        panes[targetIndex].tabId = id
        activeTabId = id
        statusMessage = "\(title) is ready. Press Start when you are ready."
    }

    public func splitActivePane() {
        guard panes.count < 2 else { return }
        let pane = TerminalPane()
        panes.append(pane)
        activePaneId = pane.id
        activeTabId = ""
    }

    public func closeTab(_ id: String) async {
        guard let index = tabIndex(id) else { return }
        let tab = tabs[index]
        if tab.lifecycle.isRunning {
            do {
                _ = try await client.send(
                    method: .terminate,
                    params: CoreParams(tabId: coreTabId(id))
                )
            } catch {
                // Ignore termination errors during close
            }
        }
        tabs.remove(at: index)
        if let paneIndex = panes.firstIndex(where: { $0.tabId == id }) {
            if panes.count > 1 {
                panes.remove(at: paneIndex)
                let replacementIndex = min(paneIndex, panes.count - 1)
                activePaneId = panes[replacementIndex].id
                activeTabId = panes[replacementIndex].tabId ?? ""
            } else {
                panes[paneIndex].tabId = nil
                activePaneId = panes[paneIndex].id
                activeTabId = ""
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
                    tabId: coreTabId(tab.id),
                    profile: tab.profile,
                    cwd: folder.path,
                    rows: 28,
                    cols: 100
                )
            )
            try apply(response, expectedTabId: tab.id, expectedCoreTabId: coreTabId(tab.id))
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
                params: CoreParams(tabId: coreTabId(tabId), dataBase64: data.base64EncodedString())
            )
            try apply(response, expectedTabId: tabId, expectedCoreTabId: coreTabId(tabId))
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
                params: CoreParams(tabId: coreTabId(tabId), rows: rows, cols: cols)
            )
            try apply(response, expectedTabId: tabId, expectedCoreTabId: coreTabId(tabId))
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
                    params: CoreParams(tabId: coreTabId(tabId))
                )
                try apply(response, expectedTabId: tabId, expectedCoreTabId: coreTabId(tabId))
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
                params: CoreParams(tabId: coreTabId(tab.id))
            )
            try apply(response, expectedTabId: tab.id, expectedCoreTabId: coreTabId(tab.id))
            statusMessage = method == .terminate ? "\(tab.title) stopped." : "Interrupted \(tab.title)."
        } catch {
            markFailed(tab.id, error: error)
        }
    }

    private func apply(
        _ response: CoreResponse,
        expectedTabId: String,
        expectedCoreTabId: String
    ) throws {
        guard response.protocolVersion == workbenchCoreProtocolVersion else {
            throw CoreClientError.protocolMismatch(response.protocolVersion)
        }
        guard let frame = response.result?.frame else {
            throw CoreClientError.transport("workbench-core returned no terminal frame")
        }
        guard frame.snapshot.tabId == expectedCoreTabId else {
            throw CoreClientError.transport(
                "workbench-core routed tab \(frame.snapshot.tabId) to \(expectedCoreTabId)"
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

    private func saveSelectedProjectTerminalWorkspace() {
        guard let selectedProjectId else { return }
        projectTerminalWorkspaces[selectedProjectId] = ProjectTerminalWorkspace(
            tabs: tabs,
            activeTabId: activeTabId,
            panes: panes,
            activePaneId: activePaneId
        )
    }

    private func coreTabId(_ tabId: String) -> String {
        guard let selectedProjectId else { return tabId }
        // The Rust protocol deliberately accepts only ASCII word-like ids.
        // Both UUID project ids and profile session ids meet that contract;
        // a dot keeps the project namespace unambiguous without introducing
        // the formerly-used, invalid `::` separator.
        return "\(selectedProjectId).\(tabId)"
    }
}
// HANDWRITE-END
