// HANDWRITE-BEGIN gap="missing-generator:unit-test:workbench-xcui-shell-journey" tracker="#2278" reason="Prove the native shell journey without agent-vision automation."
import XCTest

final class WorkbenchMacUITests: XCTestCase {
    @MainActor
    func testTerminalTabsRemainInContentChrome() throws {
        continueAfterFailure = false

        let fixtureFolder = FileManager.default.temporaryDirectory
            .appendingPathComponent("workbench-visible-tabs-ui-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: fixtureFolder, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: fixtureFolder) }

        let app = XCUIApplication()
        app.terminate()
        app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
        app.launchEnvironment["WORKBENCH_UI_TEST_FOLDER"] = fixtureFolder.path
        app.launch()
        defer { app.terminate() }

        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        let terminalTab = app.buttons["terminal.tab.claude"]
        let terminalWorkspace = app.descendants(matching: .any)["terminal.workspace"]
        XCTAssertTrue(terminalTab.waitForExistence(timeout: 5))
        XCTAssertTrue(terminalWorkspace.waitForExistence(timeout: 5))
        XCTAssertTrue(app.descendants(matching: .any)["terminal.titlebar-tabs"].exists)
        XCTAssertGreaterThanOrEqual(
            terminalTab.frame.minY,
            terminalWorkspace.frame.minY,
            "Terminal tabs must stay in app content instead of fullscreen-auto-hidden titlebar chrome."
        )
        XCTAssertLessThanOrEqual(
            terminalTab.frame.maxY,
            terminalWorkspace.frame.minY + 48,
            "The compact tab strip should remain the terminal workspace's top chrome."
        )
    }

    @MainActor
    func testFilesAuxiliaryColumnFollowsTerminalWorkspace() throws {
        continueAfterFailure = false

        let fixtureFolder = FileManager.default.temporaryDirectory
            .appendingPathComponent("workbench-files-order-ui-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: fixtureFolder, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: fixtureFolder) }

        let app = XCUIApplication()
        app.terminate()
        app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
        app.launchEnvironment["WORKBENCH_UI_TEST_FOLDER"] = fixtureFolder.path
        app.launch()
        defer { app.terminate() }

        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        let terminalTab = app.buttons["terminal.tab.claude"]
        let auxiliary = app.descendants(matching: .any)["auxiliary.column"]
        XCTAssertTrue(terminalTab.waitForExistence(timeout: 5))
        XCTAssertTrue(auxiliary.waitForExistence(timeout: 5))
        XCTAssertLessThan(
            terminalTab.frame.midX,
            auxiliary.frame.minX,
            "The primary terminal workspace must be left of the trailing Auxiliary column."
        )
    }

    @MainActor
    func testFilesAuxiliaryColumnShowsFixtureEntries() throws {
        continueAfterFailure = false

        let fixtureFolder = FileManager.default.temporaryDirectory
            .appendingPathComponent("workbench-files-ui-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: fixtureFolder, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: fixtureFolder) }
        try FileManager.default.createDirectory(at: fixtureFolder.appendingPathComponent("Sources"), withIntermediateDirectories: true)
        FileManager.default.createFile(atPath: fixtureFolder.appendingPathComponent("README.md").path, contents: Data())

        let app = XCUIApplication()
        app.terminate()
        app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
        app.launchEnvironment["WORKBENCH_UI_TEST_FOLDER"] = fixtureFolder.path
        app.launch()
        defer { app.terminate() }

        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        XCTAssertTrue(app.descendants(matching: .any)["auxiliary.column"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.descendants(matching: .any)["auxiliary.files.list"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.descendants(matching: .any)["auxiliary.file.\(fixtureFolder.appendingPathComponent("Sources").path)"].exists)
        XCTAssertTrue(app.descendants(matching: .any)["auxiliary.file.\(fixtureFolder.appendingPathComponent("README.md").path)"].exists)
    }

    /// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#unit-test
    @MainActor
    func testNativeShellJourney() throws {
        continueAfterFailure = false

        let fileManager = FileManager.default
        let fixtureFolder = fileManager.temporaryDirectory
            .appendingPathComponent("workbench-xcui-\(UUID().uuidString)", isDirectory: true)
        try fileManager.createDirectory(at: fixtureFolder, withIntermediateDirectories: true)
        defer { try? fileManager.removeItem(at: fixtureFolder) }

        let canonicalFolder = fixtureFolder.standardizedFileURL.resolvingSymlinksInPath().path
        let app = XCUIApplication()
        app.terminate()
        app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
        app.launchEnvironment["WORKBENCH_UI_TEST_FOLDER"] = fixtureFolder.path
        app.launchEnvironment["WORKBENCH_UI_TEST_ACTIVE_TAB"] = "shell"
        app.launch()
        defer { app.terminate() }

        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))

        let expectedIdleTabs = ["claude", "codex", "agy", "shell"]
        for tabId in expectedIdleTabs {
            let tab = app.buttons["terminal.tab.\(tabId)"]
            XCTAssertTrue(tab.waitForExistence(timeout: 5), "Missing default tab \(tabId)")
            XCTAssertTrue(tab.label.contains("Idle"), "\(tabId) started implicitly: \(tab.label)")
        }

        let projectList = app.descendants(matching: .any)["projects.list"]
        XCTAssertTrue(projectList.waitForExistence(timeout: 5))

        let addShell = app.buttons["terminal.add-shell"]
        XCTAssertTrue(addShell.waitForExistence(timeout: 5))
        XCTAssertGreaterThanOrEqual(addShell.frame.width, 32)
        XCTAssertGreaterThanOrEqual(addShell.frame.height, 32)

        let shellTab = app.buttons["terminal.tab.shell"]
        let start = app.buttons["terminal.launch.shell"]
        XCTAssertTrue(start.waitForExistence(timeout: 5))

        XCTAssertTrue(start.isEnabled)
        start.click()
        waitForLabel("Running", on: shellTab)

        let terminal = app.textViews["terminal.surface.shell"]
        XCTAssertTrue(terminal.waitForExistence(timeout: 5))
        terminal.click()
        let marker = "WORKBENCH_XCUI_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        terminal.typeText("printf '\(marker):%s\\n' \"$PWD\"\n")
        waitForValue(marker, on: terminal)
        waitForValue(canonicalFolder, on: terminal)

        let screenshot = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
        screenshot.name = "Workbench native shell running in selected folder"
        screenshot.lifetime = .keepAlways
        add(screenshot)

        addShell.click()
        let secondShell = app.buttons["terminal.tab.shell-2"]
        XCTAssertTrue(secondShell.waitForExistence(timeout: 5))
        XCTAssertTrue(secondShell.label.contains("Idle"))
        XCTAssertTrue(shellTab.label.contains("Running"))
        XCTAssertTrue(
            app.textViews["terminal.surface.shell"].exists,
            "The running shell's native renderer must remain mounted while another tab is selected."
        )

        let closeSecondShell = app.buttons["terminal.close-tab.shell-2"]
        XCTAssertTrue(closeSecondShell.waitForExistence(timeout: 5))
        closeSecondShell.click()
        let retainedTerminal = app.textViews["terminal.surface.shell"]
        XCTAssertTrue(retainedTerminal.waitForExistence(timeout: 5))
        waitForValue(marker, on: retainedTerminal)

        let closeShell = app.buttons["terminal.close-tab.shell"]
        XCTAssertTrue(closeShell.waitForExistence(timeout: 5))
        closeShell.click()
        XCTAssertFalse(shellTab.exists)
    }

    @MainActor
    private func waitForLabel(
        _ fragment: String,
        on element: XCUIElement,
        timeout: TimeInterval = 10
    ) {
        let predicate = NSPredicate(format: "label CONTAINS %@", fragment)
        let result = XCTWaiter.wait(
            for: [XCTNSPredicateExpectation(predicate: predicate, object: element)],
            timeout: timeout
        )
        XCTAssertEqual(result, .completed, "Element label never contained \(fragment): \(element.label)")
    }

    @MainActor
    private func waitForValue(
        _ fragment: String,
        on element: XCUIElement,
        timeout: TimeInterval = 10
    ) {
        let predicate = NSPredicate(format: "value CONTAINS %@", fragment)
        let result = XCTWaiter.wait(
            for: [XCTNSPredicateExpectation(predicate: predicate, object: element)],
            timeout: timeout
        )
        XCTAssertEqual(
            result,
            .completed,
            "Terminal value never contained \(fragment): \(String(describing: element.value))"
        )
    }
}
// HANDWRITE-END
