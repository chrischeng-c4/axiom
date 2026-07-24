// HANDWRITE-BEGIN gap="missing-generator:unit-test:d0db8335" tracker="pending-tracker" reason="Prove pane creation, explicit launch, project scoping, lifecycle text, and response routing."
import Foundation
import XCTest
@testable import WorkbenchMacCore

private actor MockCoreClient: CoreClientProtocol {
    struct Captured: Equatable {
        let method: CoreMethod
        let params: CoreParams
    }

    private var queued: [Result<CoreResponse, CoreClientError>] = []
    private var captured: [Captured] = []

    func enqueue(_ response: CoreResponse) { queued.append(.success(response)) }

    func send(method: CoreMethod, params: CoreParams) async throws -> CoreResponse {
        captured.append(Captured(method: method, params: params))
        guard !queued.isEmpty else { throw CoreClientError.transport("Mock response queue is empty") }
        return try queued.removeFirst().get()
    }

    func shutdown() async {}
    func requests() -> [Captured] { captured }
}

@MainActor
final class WorkbenchModelTests: XCTestCase {
    func testProfileSelectionCreatesIdleSessionWithoutLaunching() async {
        let client = MockCoreClient()
        let tempDir = temporaryDirectory()
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let model = WorkbenchModel(client: client, projectStore: ProjectStore(storageDirectory: tempDir))
        model.registerProject(tempDir)

        XCTAssertEqual(model.tabs, [])
        XCTAssertEqual(model.panes.count, 1)
        XCTAssertNil(model.panes[0].tabId)

        model.addTerminal(profile: .claude)

        XCTAssertEqual(model.tabs.map(\.profile), [.claude])
        XCTAssertEqual(model.activeTab?.lifecycle, .idle)
        XCTAssertEqual(model.panes.count, 1)
        XCTAssertEqual(model.panes[0].tabId, "claude")
        let requests = await client.requests()
        XCTAssertTrue(requests.isEmpty, "choosing a profile must not launch a PTY")
    }

    func testSecondProfileUsesRightPaneAndCloseRestoresOnePane() async {
        let client = MockCoreClient()
        let tempDir = temporaryDirectory()
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let model = WorkbenchModel(client: client, projectStore: ProjectStore(storageDirectory: tempDir))
        model.registerProject(tempDir)

        model.addTerminal(profile: .claude)
        model.addTerminal(profile: .shell)

        XCTAssertEqual(model.panes.count, 2)
        XCTAssertEqual(model.panes.map(\.tabId), ["claude", "shell"])
        XCTAssertEqual(model.activeTabId, "shell")
        let requests = await client.requests()
        XCTAssertTrue(requests.isEmpty)

        await model.closeTab("shell")
        XCTAssertEqual(model.panes.count, 1)
        XCTAssertEqual(model.panes[0].tabId, "claude")
        XCTAssertEqual(model.activeTabId, "claude")
    }

    func testProjectSwitchRestoresIndependentPaneLayoutsAndSessions() async {
        let client = MockCoreClient()
        let tempDir = temporaryDirectory()
        let firstProject = tempDir.appendingPathComponent("first", isDirectory: true)
        let secondProject = tempDir.appendingPathComponent("second", isDirectory: true)
        try? FileManager.default.createDirectory(at: firstProject, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(at: secondProject, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let model = WorkbenchModel(client: client, projectStore: ProjectStore(storageDirectory: tempDir))
        model.registerProject(firstProject)
        model.registerProject(secondProject)
        let first = try! XCTUnwrap(model.projects.first)
        let second = try! XCTUnwrap(model.projects.last)

        model.selectProject(first.id)
        model.addTerminal(profile: .claude)
        await client.enqueue(response(tabId: "\(first.id).claude", profile: .claude, running: true, cwd: firstProject.path, sequence: 1))
        await model.startActiveTab()
        model.addTerminal(profile: .shell)

        model.selectProject(second.id)
        XCTAssertEqual(model.selectedProjectId, second.id)
        XCTAssertEqual(model.tabs, [])
        XCTAssertEqual(model.panes.count, 1)
        XCTAssertNil(model.panes[0].tabId)

        model.addTerminal(profile: .codex)
        model.selectProject(first.id)
        XCTAssertEqual(model.panes.map(\.tabId), ["claude", "shell"])
        XCTAssertEqual(model.tabs.first(where: { $0.id == "claude" })?.lifecycle, .running)
        XCTAssertEqual(model.tabs.first(where: { $0.id == "shell" })?.lifecycle, .idle)

        model.selectProject(second.id)
        XCTAssertEqual(model.panes.map(\.tabId), ["codex"])
        XCTAssertEqual(model.activeTab?.profile, .codex)
        XCTAssertEqual(model.activeTab?.lifecycle, .idle)
    }

    func testResponsesRemainSessionScopedAndLifecycleTextIsVisible() async {
        let client = MockCoreClient()
        let tempDir = temporaryDirectory()
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let model = WorkbenchModel(client: client, projectStore: ProjectStore(storageDirectory: tempDir))
        model.registerProject(tempDir)
        let projectId = try! XCTUnwrap(model.selectedProjectId)
        model.addTerminal(profile: .claude)
        await client.enqueue(response(tabId: "\(projectId).claude", profile: .claude, running: true, cwd: tempDir.path, sequence: 1, output: Data("CLAUDE".utf8)))
        await model.startActiveTab()
        XCTAssertEqual(model.activeTab?.lifecycle.label, "Running")
        XCTAssertEqual(String(data: model.activeTab?.output ?? Data(), encoding: .utf8), "CLAUDE")

        await client.enqueue(response(tabId: "\(projectId).codex", profile: .codex, running: false, cwd: tempDir.path, sequence: 2))
        await model.pollRunningTabs()

        guard case let .failed(message) = model.activeTab?.lifecycle else { return XCTFail("mismatched response must fail only the addressed session") }
        XCTAssertTrue(message.contains("routed tab"))
        XCTAssertEqual(model.activeTab?.accessibilityLabel, "Claude Code, Needs attention")
    }

    func testProjectsPersistIndependentlyAndRemoveOnlyMetadata() async {
        let client = MockCoreClient()
        let tempDir = temporaryDirectory()
        let firstProject = tempDir.appendingPathComponent("first", isDirectory: true)
        let secondProject = tempDir.appendingPathComponent("second", isDirectory: true)
        try? FileManager.default.createDirectory(at: firstProject, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(at: secondProject, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let store = ProjectStore(storageDirectory: tempDir)
        let model = WorkbenchModel(client: client, projectStore: store)

        model.registerProject(firstProject)
        model.registerProject(secondProject)
        let first = try! XCTUnwrap(model.projects.first)
        XCTAssertTrue(FileManager.default.fileExists(atPath: store.projectDirectory(first.id).appendingPathComponent("project.json").path))
        XCTAssertEqual(WorkbenchModel(client: client, projectStore: store).projects.map(\.rootPath), [firstProject.path, secondProject.path])

        model.removeProject(first.id)
        XCTAssertFalse(FileManager.default.fileExists(atPath: store.projectDirectory(first.id).path))
        XCTAssertTrue(FileManager.default.fileExists(atPath: firstProject.path))
    }

    func testNativeClientUsesSwiftTermAndPaneProfileMenuWithoutWebView() throws {
        let packageRoot = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let package = try String(contentsOf: packageRoot.appendingPathComponent("Package.swift"), encoding: .utf8)
        let terminal = try String(contentsOf: packageRoot.appendingPathComponent("Sources/WorkbenchMac/TerminalSurface.swift"), encoding: .utf8)
        let view = try String(contentsOf: packageRoot.appendingPathComponent("Sources/WorkbenchMac/WorkbenchView.swift"), encoding: .utf8)

        XCTAssertTrue(package.contains("SwiftTerm"))
        XCTAssertTrue(terminal.contains("NSViewRepresentable"))
        XCTAssertTrue(terminal.contains("TerminalViewDelegate"))
        XCTAssertTrue(terminal.contains("feed(byteArray:"))
        XCTAssertFalse(terminal.contains("WKWebView"))
        XCTAssertFalse(view.contains("WebKit"))
        XCTAssertTrue(view.contains("Add terminal profile"))
        XCTAssertTrue(view.contains("Split right"))
        XCTAssertFalse(view.contains("terminal.titlebar-tabs"))
    }

    private func temporaryDirectory() -> URL {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        return directory
    }

    private func response(tabId: String, profile: TerminalProfile, running: Bool, cwd: String, sequence: UInt64, output: Data = Data()) -> CoreResponse {
        CoreResponse(requestId: sequence + 1, ok: true, result: CoreResult(kind: "session", frame: CoreTerminalFrame(snapshot: CoreTerminalSnapshot(tabId: tabId, profile: profile, label: profile.label, running: running, processId: running ? 1234 : nil, exitCode: nil, activeCwd: cwd), sequence: sequence, outputBase64: output.base64EncodedString())))
    }
}
// HANDWRITE-END
