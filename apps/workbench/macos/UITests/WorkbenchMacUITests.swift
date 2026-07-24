// HANDWRITE-BEGIN gap="missing-generator:unit-test:workbench-xcui-pane-journey" tracker="#2493" reason="Prove pane-first chrome and the project-scoped Files auxiliary column without agent-vision automation."
import XCTest

final class WorkbenchMacUITests: XCTestCase {
    private var uiTestStateRoots: [URL] = []

    override func tearDownWithError() throws {
        for root in uiTestStateRoots {
            try? FileManager.default.removeItem(at: root)
        }
        uiTestStateRoots.removeAll()
        try super.tearDownWithError()
    }

    @MainActor
    func testPaneToolbarRemainsInContentChrome() throws {
        let fixtureFolder = try makeFixtureFolder()
        defer { try? FileManager.default.removeItem(at: fixtureFolder) }
        let app = launch(fixtureFolder)
        defer { app.terminate() }

        let workspace = app.descendants(matching: .any)["terminal.workspace"]
        let toolbar = app.descendants(matching: .any)["terminal.pane-toolbar"]
        let addProfile = app.buttons["terminal.add-profile"]
        XCTAssertTrue(workspace.waitForExistence(timeout: 5))
        XCTAssertTrue(toolbar.waitForExistence(timeout: 5))
        XCTAssertTrue(addProfile.waitForExistence(timeout: 5))
        XCTAssertFalse(app.descendants(matching: .any)["terminal.titlebar-tabs"].exists)
        XCTAssertGreaterThanOrEqual(toolbar.frame.minY, workspace.frame.minY)
        XCTAssertLessThanOrEqual(toolbar.frame.maxY, workspace.frame.minY + 44)
        capture("pane-toolbar-in-content-chrome", app: app)
    }

    @MainActor
    func testFilesAuxiliaryColumnFollowsTerminalWorkspace() throws {
        let fixtureFolder = try makeFixtureFolder()
        defer { try? FileManager.default.removeItem(at: fixtureFolder) }
        let app = launch(fixtureFolder)
        defer { app.terminate() }

        let toolbar = app.descendants(matching: .any)["terminal.pane-toolbar"]
        let auxiliary = app.descendants(matching: .any)["auxiliary.column"]
        XCTAssertTrue(toolbar.waitForExistence(timeout: 5))
        XCTAssertTrue(auxiliary.waitForExistence(timeout: 5))
        XCTAssertLessThan(toolbar.frame.midX, auxiliary.frame.minX)
        capture("terminal-before-auxiliary-column", app: app)
    }

    @MainActor
    func testFilesAuxiliaryColumnShowsFixtureEntries() throws {
        let fixtureFolder = try makeFixtureFolder()
        try FileManager.default.createDirectory(at: fixtureFolder.appendingPathComponent("Sources"), withIntermediateDirectories: true)
        FileManager.default.createFile(atPath: fixtureFolder.appendingPathComponent("README.md").path, contents: Data())
        defer { try? FileManager.default.removeItem(at: fixtureFolder) }
        let app = launch(fixtureFolder)
        defer { app.terminate() }

        XCTAssertTrue(app.descendants(matching: .any)["auxiliary.files.list"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.descendants(matching: .any)["auxiliary.file.\(fixtureFolder.appendingPathComponent("Sources").path)"].exists)
        XCTAssertTrue(app.descendants(matching: .any)["auxiliary.file.\(fixtureFolder.appendingPathComponent("README.md").path)"].exists)
        capture("auxiliary-files-fixture-entries", app: app)
    }

    @MainActor
    func testPaneToolbarOffersAddAndExplicitSplitActions() throws {
        let fixtureFolder = try makeFixtureFolder()
        defer { try? FileManager.default.removeItem(at: fixtureFolder) }
        let app = launch(fixtureFolder)
        defer { app.terminate() }

        let addProfile = app.buttons["terminal.add-profile"]
        XCTAssertTrue(addProfile.waitForExistence(timeout: 5))
        addProfile.click()
        let claude = app.menuItems["Claude Code"]
        XCTAssertTrue(claude.waitForExistence(timeout: 3))
        claude.click()
        XCTAssertTrue(
            app.descendants(matching: .any)["terminal.idle.claude"]
                .waitForExistence(timeout: 5)
        )

        addProfile.click()
        XCTAssertTrue(app.menuItems["Split Right"].waitForExistence(timeout: 3))
        XCTAssertTrue(app.menuItems["Split Down"].exists)
        app.typeKey(.escape, modifierFlags: [])
        capture("occupied-pane-split-actions", app: app)
    }

    @MainActor
    private func launch(_ fixtureFolder: URL) -> XCUIApplication {
        let app = XCUIApplication()
        app.terminate()
        let stateRoot = repositoryRoot
            .appendingPathComponent(".axiom-workbench/test-artifacts/ui-tests", isDirectory: true)
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        uiTestStateRoots.append(stateRoot)
        app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
        app.launchEnvironment["WORKBENCH_UI_TEST_FOLDER"] = fixtureFolder.path
        app.launchEnvironment["WORKBENCH_UI_TEST_STATE_ROOT"] = stateRoot.path
        app.launch()
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        return app
    }

    @MainActor
    private func capture(_ name: String, app: XCUIApplication) {
        let attachment = XCTAttachment(screenshot: app.screenshot())
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }

    private var repositoryRoot: URL {
        var root = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 {
            root.deleteLastPathComponent()
        }
        return root
    }

    private func makeFixtureFolder() throws -> URL {
        let folder = FileManager.default.temporaryDirectory
            .appendingPathComponent("workbench-pane-ui-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        return folder
    }
}
// HANDWRITE-END
