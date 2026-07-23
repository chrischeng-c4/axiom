// HANDWRITE-BEGIN gap="missing-generator:unit-test:d0db8335" tracker="pending-tracker" reason="Prove default ordering, no implicit launch, plus behavior, request scoping, selected cwd, lifecycle text, and response routing."
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

    func enqueue(_ response: CoreResponse) {
        queued.append(.success(response))
    }

    func send(method: CoreMethod, params: CoreParams) async throws -> CoreResponse {
        captured.append(Captured(method: method, params: params))
        guard !queued.isEmpty else {
            throw CoreClientError.transport("Mock response queue is empty")
        }
        return try queued.removeFirst().get()
    }

    func shutdown() async {}

    func requests() -> [Captured] {
        captured
    }
}

@MainActor
final class WorkbenchModelTests: XCTestCase {
    func testDefaultTabsAreOrderedAndIdle() async {
        await defaultTabsAreOrderedAndIdle()
    }

    func testAddingShellTabSelectsWithoutLaunching() async {
        await addingShellTabSelectsWithoutLaunching()
    }

    func testResponsesRemainTabScopedAndLifecycleTextIsVisible() async {
        await responsesRemainTabScopedAndLifecycleTextIsVisible()
    }

    func testNativeClientUsesSwiftTermWithoutWebView() throws {
        try nativeClientUsesSwiftTermWithoutWebView()
    }

    func testClosingTabRemovesTabAndSelectsAdjacent() async {
        await closingTabRemovesTabAndSelectsAdjacent()
    }

    func testProjectsPersistIndependentlyAndRemoveOnlyMetadata() async {
        await projectsPersistIndependentlyAndRemoveOnlyMetadata()
    }

    func testProjectsKeepIndependentTerminalWorkspaces() async {
        await projectsKeepIndependentTerminalWorkspaces()
    }

    func closingTabRemovesTabAndSelectsAdjacent() async {
        let client = MockCoreClient()
        let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let store = ProjectStore(storageDirectory: tempDir)
        let model = WorkbenchModel(client: client, projectStore: store)

        XCTAssertEqual(model.tabs.count, 4)
        XCTAssertEqual(model.activeTabId, "claude")

        await model.closeTab("claude")
        XCTAssertEqual(model.tabs.count, 3)
        XCTAssertEqual(model.activeTabId, "codex")

        try? FileManager.default.removeItem(at: tempDir)
    }

    func projectsPersistIndependentlyAndRemoveOnlyMetadata() async {
        let client = MockCoreClient()
        let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let firstProject = tempDir.appendingPathComponent("first", isDirectory: true)
        let secondProject = tempDir.appendingPathComponent("second", isDirectory: true)
        try? FileManager.default.createDirectory(at: firstProject, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(at: secondProject, withIntermediateDirectories: true)
        let store = ProjectStore(storageDirectory: tempDir)
        let model = WorkbenchModel(client: client, projectStore: store)

        model.registerProject(firstProject)
        model.registerProject(secondProject)
        XCTAssertEqual(model.projects.map(\.rootPath), [firstProject.path, secondProject.path])
        XCTAssertEqual(model.selectedFolder?.path, secondProject.path)
        let first = try! XCTUnwrap(model.projects.first)
        XCTAssertTrue(FileManager.default.fileExists(atPath: store.projectDirectory(first.id).appendingPathComponent("project.json").path))

        let restored = WorkbenchModel(client: client, projectStore: store)
        XCTAssertEqual(restored.projects.map(\.rootPath), [firstProject.path, secondProject.path])

        model.removeProject(first.id)
        XCTAssertFalse(FileManager.default.fileExists(atPath: store.projectDirectory(first.id).path))
        XCTAssertTrue(FileManager.default.fileExists(atPath: firstProject.path), "removing a project must not delete its files")
        XCTAssertEqual(model.projects.map(\.rootPath), [secondProject.path])

        try? FileManager.default.removeItem(at: tempDir)
    }

    func projectsKeepIndependentTerminalWorkspaces() async {
        let client = MockCoreClient()
        let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let firstProject = tempDir.appendingPathComponent("first", isDirectory: true)
        let secondProject = tempDir.appendingPathComponent("second", isDirectory: true)
        try? FileManager.default.createDirectory(at: firstProject, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(at: secondProject, withIntermediateDirectories: true)
        FileManager.default.createFile(
            atPath: secondProject.appendingPathComponent("project-marker.txt").path,
            contents: Data()
        )
        let model = WorkbenchModel(
            client: client,
            projectStore: ProjectStore(storageDirectory: tempDir)
        )
        model.registerProject(firstProject)
        model.registerProject(secondProject)
        let first = try! XCTUnwrap(model.projects.first)
        let second = try! XCTUnwrap(model.projects.last)

        model.selectProject(first.id)
        await client.enqueue(
            response(
                tabId: "\(first.id)::claude",
                profile: .claude,
                running: true,
                cwd: firstProject.path,
                sequence: 0
            )
        )
        await model.startActiveTab()
        let requestsBeforeSwitch = await client.requests()

        model.selectProject(second.id)

        XCTAssertEqual(model.selectedWorkspace?.project.id, second.id)
        XCTAssertEqual(model.selectedFolder?.path, secondProject.path)
        if case let .available(entries, _) = model.projectFileListing {
            XCTAssertTrue(entries.contains(where: { $0.name == "project-marker.txt" }))
        } else {
            XCTFail("selected project should expose its Files listing")
        }
        XCTAssertEqual(model.activeTab?.lifecycle, .idle)
        XCTAssertEqual(model.activeTab?.output, Data())
        let retainedA = model.mountedTerminalTabs.first {
            $0.projectId == first.id && $0.tab.id == "claude"
        }
        XCTAssertEqual(retainedA?.tab.lifecycle, .running)
        let requestsAfterSwitch = await client.requests()
        XCTAssertEqual(requestsAfterSwitch, requestsBeforeSwitch)

        model.selectProject(first.id)
        XCTAssertEqual(model.activeTab?.activeCwd, firstProject.path)
        XCTAssertEqual(model.activeTab?.lifecycle, .running)

        try? FileManager.default.removeItem(at: tempDir)
    }

    func defaultTabsAreOrderedAndIdle() async {
        let client = MockCoreClient()
        let model = WorkbenchModel(client: client)

        XCTAssertEqual(model.tabs.map(\.title), ["Claude Code", "Codex", "AGY", "Shell"])
        XCTAssertEqual(model.tabs.map(\.profile), [.claude, .codex, .agy, .shell])
        XCTAssertEqual(model.tabs.map(\.lifecycle), [.idle, .idle, .idle, .idle])
        XCTAssertEqual(model.activeTabId, "claude")

        model.registerProject(URL(fileURLWithPath: "/tmp", isDirectory: true))
        model.selectTab("codex")
        model.selectDefaultTab(at: 2)
        let selectionRequests = await client.requests()
        XCTAssertTrue(selectionRequests.isEmpty, "selection must not contact or start the sidecar")
        XCTAssertEqual(model.tabs.map(\.lifecycle), [.idle, .idle, .idle, .idle])
    }

    func addingShellTabSelectsWithoutLaunching() async {
        let client = MockCoreClient()
        let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let model = WorkbenchModel(client: client, projectStore: ProjectStore(storageDirectory: tempDir))
        defer { try? FileManager.default.removeItem(at: tempDir) }
        model.addShellTab()

        XCTAssertEqual(model.tabs.last?.id, "shell-2")
        XCTAssertEqual(model.tabs.last?.title, "Shell 2")
        XCTAssertEqual(model.tabs.last?.lifecycle, .idle)
        XCTAssertEqual(model.activeTabId, "shell-2")
        let additionRequests = await client.requests()
        XCTAssertTrue(additionRequests.isEmpty)

        let folder = URL(fileURLWithPath: "/tmp", isDirectory: true)
        model.registerProject(folder)
        await client.enqueue(
            response(
                tabId: "\(model.selectedProjectId!)::shell-2",
                profile: .shell,
                running: true,
                cwd: folder.resolvingSymlinksInPath().path,
                sequence: 0
            )
        )
        await model.startActiveTab()

        let requests = await client.requests()
        XCTAssertEqual(requests.count, 1)
        XCTAssertEqual(requests[0].method, .launch)
        XCTAssertEqual(requests[0].params.tabId, "\(model.selectedProjectId!)::shell-2")
        XCTAssertEqual(requests[0].params.profile, .shell)
        XCTAssertEqual(requests[0].params.cwd, folder.resolvingSymlinksInPath().path)
        XCTAssertEqual(model.activeTab?.lifecycle, .running)
    }

    func responsesRemainTabScopedAndLifecycleTextIsVisible() async {
        let client = MockCoreClient()
        let model = WorkbenchModel(client: client)
        let folder = URL(fileURLWithPath: "/tmp", isDirectory: true).resolvingSymlinksInPath()
        model.registerProject(folder)
        await client.enqueue(
            response(
                tabId: "\(model.selectedProjectId!)::claude",
                profile: .claude,
                running: true,
                cwd: folder.path,
                sequence: 1,
                output: Data("CLAUDE".utf8)
            )
        )
        await model.startActiveTab()
        XCTAssertEqual(model.tabs[0].lifecycle.label, "Running")
        XCTAssertEqual(String(data: model.tabs[0].output, encoding: .utf8), "CLAUDE")

        await client.enqueue(
            response(
                tabId: "\(model.selectedProjectId!)::codex",
                profile: .codex,
                running: false,
                cwd: folder.path,
                sequence: 1,
                output: Data("WRONG".utf8)
            )
        )
        await model.pollRunningTabs()

        guard case let .failed(message) = model.tabs[0].lifecycle else {
            return XCTFail("mismatched response must fail the addressed tab")
        }
        XCTAssertTrue(message.contains("routed tab"))
        XCTAssertEqual(model.tabs[1].lifecycle, .idle)
        XCTAssertEqual(model.tabs[1].output, Data())
        XCTAssertEqual(model.tabs[0].accessibilityLabel, "Claude Code, Needs attention")
    }

    func nativeClientUsesSwiftTermWithoutWebView() throws {
        let packageRoot = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        let package = try String(
            contentsOf: packageRoot.appendingPathComponent("Package.swift"),
            encoding: .utf8
        )
        let terminal = try String(
            contentsOf: packageRoot.appendingPathComponent("Sources/WorkbenchMac/TerminalSurface.swift"),
            encoding: .utf8
        )
        let view = try String(
            contentsOf: packageRoot.appendingPathComponent("Sources/WorkbenchMac/WorkbenchView.swift"),
            encoding: .utf8
        )

        XCTAssertTrue(package.contains("SwiftTerm"))
        XCTAssertTrue(terminal.contains("NSViewRepresentable"))
        XCTAssertTrue(terminal.contains("TerminalViewDelegate"))
        XCTAssertTrue(terminal.contains("feed(byteArray:"))
        XCTAssertFalse(terminal.contains("WKWebView"))
        XCTAssertFalse(view.contains("WebKit"))
        XCTAssertTrue(view.contains("Add shell terminal tab"))
        XCTAssertTrue(view.contains("accessibilityLabel"))
    }

    private func response(
        tabId: String,
        profile: TerminalProfile,
        running: Bool,
        cwd: String,
        sequence: UInt64,
        output: Data = Data(),
        exitCode: UInt32? = nil
    ) -> CoreResponse {
        CoreResponse(
            requestId: sequence + 1,
            ok: true,
            result: CoreResult(
                kind: "session",
                frame: CoreTerminalFrame(
                    snapshot: CoreTerminalSnapshot(
                        tabId: tabId,
                        profile: profile,
                        label: profile.label,
                        running: running,
                        processId: running ? 1234 : nil,
                        exitCode: exitCode,
                        activeCwd: cwd
                    ),
                    sequence: sequence,
                    outputBase64: output.base64EncodedString()
                )
            )
        )
    }
}
// HANDWRITE-END
