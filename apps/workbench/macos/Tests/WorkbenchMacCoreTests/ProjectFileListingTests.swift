// HANDWRITE-BEGIN gap="missing-generator:unit-test:a388f270" tracker="#2444" reason="Verify the bounded, direct-child, read-only Files listing used by Workbench Beta."
import Foundation
import XCTest
@testable import WorkbenchMacCore

final class ProjectFileListingTests: XCTestCase {
    func testVisibleEntriesAreSortedAndBounded() throws {
        let root = try makeRoot()
        defer { try? FileManager.default.removeItem(at: root) }
        try FileManager.default.createDirectory(at: root.appendingPathComponent("Zoo"), withIntermediateDirectories: true)
        try FileManager.default.createDirectory(at: root.appendingPathComponent("alpha"), withIntermediateDirectories: true)
        FileManager.default.createFile(atPath: root.appendingPathComponent("zeta.swift").path, contents: Data())
        FileManager.default.createFile(atPath: root.appendingPathComponent("README.md").path, contents: Data())
        FileManager.default.createFile(atPath: root.appendingPathComponent(".hidden").path, contents: Data())
        try FileManager.default.createSymbolicLink(at: root.appendingPathComponent("outside"), withDestinationURL: URL(fileURLWithPath: "/tmp"))

        let state = ProjectFileListing().load(root: root)
        guard case let .available(entries, isTruncated) = state else {
            return XCTFail("expected visible entries")
        }
        XCTAssertFalse(isTruncated)
        XCTAssertEqual(entries.map(\.name), ["alpha", "Zoo", "README.md", "zeta.swift"])
        XCTAssertEqual(entries.map(\.kind), [.directory, .directory, .file, .file])
        XCTAssertFalse(entries.contains { $0.name == ".hidden" || $0.name == "outside" })
    }

    func testUnavailableAndEmptyRootsRemainExplicit() throws {
        XCTAssertEqual(ProjectFileListing().load(root: nil), .noProject)
        let missing = URL(fileURLWithPath: "/tmp/workbench-missing-\(UUID().uuidString)", isDirectory: true)
        XCTAssertEqual(
            ProjectFileListing().load(root: missing),
            .unavailable("The selected project folder is unavailable.")
        )

        let root = try makeRoot()
        defer { try? FileManager.default.removeItem(at: root) }
        XCTAssertEqual(ProjectFileListing().load(root: root), .empty)
    }

    func testListingCapsVisibleEntries() throws {
        let root = try makeRoot()
        defer { try? FileManager.default.removeItem(at: root) }
        for index in 0 ... ProjectFileListing.maximumEntries {
            FileManager.default.createFile(
                atPath: root.appendingPathComponent(String(format: "file-%03d", index)).path,
                contents: Data()
            )
        }

        let state = ProjectFileListing().load(root: root)
        guard case let .available(entries, isTruncated) = state else {
            return XCTFail("expected capped entries")
        }
        XCTAssertEqual(entries.count, ProjectFileListing.maximumEntries)
        XCTAssertTrue(isTruncated)
    }

    private func makeRoot() throws -> URL {
        let root = FileManager.default.temporaryDirectory
            .appendingPathComponent("workbench-files-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        return root
    }
}
// HANDWRITE-END
