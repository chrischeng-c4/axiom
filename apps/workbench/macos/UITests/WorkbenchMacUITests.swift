// HANDWRITE-BEGIN gap="missing-generator:unit-test:workbench-xcui-pane-journey" tracker="#2493" reason="Prove pane-first chrome and the project-scoped Files auxiliary column without agent-vision automation."
import XCTest

final class WorkbenchMacUITests: XCTestCase {
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
    }

    @MainActor
    private func launch(_ fixtureFolder: URL) -> XCUIApplication {
        let app = XCUIApplication()
        app.terminate()
        app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
        app.launchEnvironment["WORKBENCH_UI_TEST_FOLDER"] = fixtureFolder.path
        app.launch()
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        return app
    }

    private func makeFixtureFolder() throws -> URL {
        let folder = FileManager.default.temporaryDirectory
            .appendingPathComponent("workbench-pane-ui-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        return folder
    }
}
// HANDWRITE-END
