// HANDWRITE-BEGIN gap="missing-generator:logic:workbench-project-metadata" tracker="pending-tracker" reason="Persist user-registered Workbench projects as independently removable metadata directories."
import Foundation

public struct RegisteredProject: Codable, Identifiable, Equatable, Sendable {
    public let id: String
    public let displayName: String
    public let rootPath: String
    public let addedAt: Date

    public init(id: String, displayName: String, rootPath: String, addedAt: Date = Date()) {
        self.id = id
        self.displayName = displayName
        self.rootPath = rootPath
        self.addedAt = addedAt
    }
}

/// Filesystem-backed registry for the projects a user intentionally adds to Workbench.
///
/// Every project owns one metadata directory under
/// `~/.axiom-workbench/projects/<project-id>/project.json`; the repository itself
/// is never modified or deleted by this store.
public final class ProjectStore: @unchecked Sendable {
    private let rootDirectory: URL
    private let fileManager: FileManager

    public init(storageDirectory: URL? = nil, fileManager: FileManager = .default) {
        self.fileManager = fileManager
        rootDirectory = storageDirectory
            ?? fileManager.homeDirectoryForCurrentUser.appendingPathComponent(".axiom-workbench")
    }

    public func load() -> [RegisteredProject] {
        let projectsDirectory = rootDirectory.appendingPathComponent("projects", isDirectory: true)
        guard let entries = try? fileManager.contentsOfDirectory(
            at: projectsDirectory,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles]
        ) else {
            return []
        }
        return entries.compactMap { directory in
            let metadata = directory.appendingPathComponent("project.json")
            guard let data = try? Data(contentsOf: metadata) else { return nil }
            return try? JSONDecoder().decode(RegisteredProject.self, from: data)
        }
        .sorted { $0.addedAt < $1.addedAt }
    }

    @discardableResult
    public func register(url: URL) -> RegisteredProject {
        let resolved = url.standardizedFileURL.resolvingSymlinksInPath()
        if let existing = load().first(where: { $0.rootPath == resolved.path }) {
            return existing
        }

        let project = RegisteredProject(
            id: UUID().uuidString.lowercased(),
            displayName: resolved.lastPathComponent,
            rootPath: resolved.path
        )
        let directory = projectDirectory(project.id)
        try? fileManager.createDirectory(at: directory, withIntermediateDirectories: true)
        if let data = try? JSONEncoder().encode(project) {
            try? data.write(to: directory.appendingPathComponent("project.json"), options: .atomic)
        }
        return project
    }

    /// Unregisters a project by removing only its Workbench metadata directory.
    public func remove(id: String) {
        try? fileManager.removeItem(at: projectDirectory(id))
    }

    public func projectDirectory(_ id: String) -> URL {
        rootDirectory
            .appendingPathComponent("projects", isDirectory: true)
            .appendingPathComponent(id, isDirectory: true)
    }
}
// HANDWRITE-END
