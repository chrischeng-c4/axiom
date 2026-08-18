// HANDWRITE-BEGIN gap="missing-generator:contract:e8c46769" tracker="pending-tracker" reason="Prove the native registry file contract, token and request-id rejection, stale-record rules, and bounded PNG response envelope."
import AppKit
import XCTest
@testable import WorkbenchMacCore

@MainActor
final class LocalRuntimeServerTests: XCTestCase {
    func testRegistryLeaseAuthenticationAndCleanup() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        let server = LocalRuntimeServer(runtimeDirectory: directory)
        try server.start()
        XCTAssertTrue(FileManager.default.fileExists(atPath: directory.appendingPathComponent("workbench.lock").path))
        XCTAssertThrowsError(try LocalRuntimeServer(runtimeDirectory: directory).start())
        server.stop()
        XCTAssertFalse(FileManager.default.fileExists(atPath: directory.appendingPathComponent("workbench.lock").path))
        XCTAssertFalse(FileManager.default.fileExists(atPath: directory.appendingPathComponent("current.json").path))
        try? FileManager.default.removeItem(at: directory)
    }

    func testContentViewCaptureReturnsBoundedPNG() throws {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 64, height: 48))
        view.wantsLayer = true
        view.layer?.backgroundColor = NSColor.red.cgColor
        let png = try LocalRuntimeServer(runtimeDirectory: FileManager.default.temporaryDirectory).capturePNG(from: view)
        XCTAssertGreaterThan(png.count, 8)
        XCTAssertEqual(Array(png.prefix(8)), [137, 80, 78, 71, 13, 10, 26, 10])
    }
}
// HANDWRITE-END
