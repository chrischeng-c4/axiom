// HANDWRITE-BEGIN gap="missing-generator:logic:workbench-runtime-profile-tests" tracker="#2445" reason="Verify isolated app channel identity, roots, and build script product selection."
import Foundation
import XCTest
@testable import WorkbenchMacCore

final class WorkbenchRuntimeProfileTests: XCTestCase {
    func testStableAndBetaProductsAreDistinct() {
        XCTAssertEqual(WorkbenchRuntimeProfile.stable.productName, "Axiom Workbench")
        XCTAssertEqual(WorkbenchRuntimeProfile.stable.bundleIdentifier, "com.axiom.workbench")
        XCTAssertEqual(WorkbenchRuntimeProfile.beta.productName, "Axiom Workbench Beta")
        XCTAssertEqual(WorkbenchRuntimeProfile.beta.bundleIdentifier, "com.axiom.workbench.beta")
    }

    func testProfileRootsDoNotOverlap() {
        let fileManager = FileManager.default
        let stable = WorkbenchRuntimeProfile.stable.stateRoot(fileManager: fileManager)
        let beta = WorkbenchRuntimeProfile.beta.stateRoot(fileManager: fileManager)
        XCTAssertNotEqual(stable, beta)
        XCTAssertNotEqual(
            WorkbenchRuntimeProfile.stable.runtimeRoot(fileManager: fileManager),
            WorkbenchRuntimeProfile.beta.runtimeRoot(fileManager: fileManager)
        )
        XCTAssertNotEqual(
            WorkbenchRuntimeProfile.stable.logFile(fileManager: fileManager),
            WorkbenchRuntimeProfile.beta.logFile(fileManager: fileManager)
        )
    }

    func testBuildSkillsAreProductScoped() throws {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let stable = try String(contentsOf: root.appendingPathComponent(".agents/skills/workbench-build-stable/scripts/build.sh"))
        let beta = try String(contentsOf: root.appendingPathComponent(".agents/skills/workbench-build-beta/scripts/build.sh"))
        XCTAssertTrue(stable.contains("-configuration Release"))
        XCTAssertTrue(stable.contains("Axiom Workbench.app"))
        XCTAssertFalse(stable.contains("Axiom Workbench Beta.app"))
        XCTAssertTrue(beta.contains("-configuration Debug"))
        XCTAssertTrue(beta.contains("Axiom Workbench Beta.app"))
        XCTAssertFalse(beta.contains("Axiom Workbench.app\""))
    }
}
// HANDWRITE-END
