// HANDWRITE-BEGIN gap="missing-generator:logic:c969a33f" tracker="#2444" reason="Expose a bounded, read-only direct-child listing for the Beta auxiliary Files section."
import Foundation

public struct ProjectFileEntry: Identifiable, Equatable, Sendable {
    public enum Kind: Equatable, Sendable {
        case directory
        case file
    }

    public let id: String
    public let name: String
    public let path: String
    public let kind: Kind

    public init(url: URL, kind: Kind) {
        id = url.path
        name = url.lastPathComponent
        path = url.path
        self.kind = kind
    }
}

public enum ProjectFileListingState: Equatable, Sendable {
    case noProject
    case available(entries: [ProjectFileEntry], isTruncated: Bool)
    case empty
    case unavailable(String)
}

/// Read-only direct-child listing for a registered project root.
///
/// This deliberately never reads file contents, recurses into directories, or
/// follows symbolic links. It is presentation data for the Beta-only
/// auxiliary column, not a file-management API.
public struct ProjectFileListing {
    public static let maximumEntries = 200

    private let fileManager: FileManager

    public init(fileManager: FileManager = .default) {
        self.fileManager = fileManager
    }

    public func load(root: URL?) -> ProjectFileListingState {
        guard let root else { return .noProject }
        let resolvedRoot = root.standardizedFileURL.resolvingSymlinksInPath()
        var rootIsDirectory: ObjCBool = false
        guard fileManager.fileExists(atPath: resolvedRoot.path, isDirectory: &rootIsDirectory),
              rootIsDirectory.boolValue
        else {
            return .unavailable("The selected project folder is unavailable.")
        }

        guard let enumerator = fileManager.enumerator(
            at: resolvedRoot,
            includingPropertiesForKeys: [.isDirectoryKey, .isSymbolicLinkKey, .isHiddenKey],
            options: [.skipsHiddenFiles, .skipsSubdirectoryDescendants, .skipsPackageDescendants],
            errorHandler: { _, _ in false }
        ) else {
            return .unavailable("Workbench could not read the selected project folder.")
        }

        var entries: [ProjectFileEntry] = []
        var isTruncated = false
        while let candidate = enumerator.nextObject() as? URL {
            if entries.count >= Self.maximumEntries {
                isTruncated = true
                break
            }
            guard let values = try? candidate.resourceValues(forKeys: [.isDirectoryKey, .isSymbolicLinkKey, .isHiddenKey]),
                  values.isHidden != true,
                  values.isSymbolicLink != true
            else {
                continue
            }
            let kind: ProjectFileEntry.Kind = values.isDirectory == true ? .directory : .file
            entries.append(ProjectFileEntry(url: candidate, kind: kind))
        }

        entries.sort {
            let leftRank = $0.kind == .directory ? 0 : 1
            let rightRank = $1.kind == .directory ? 0 : 1
            if leftRank != rightRank { return leftRank < rightRank }
            return $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending
        }
        return entries.isEmpty ? .empty : .available(entries: entries, isTruncated: isTruncated)
    }
}
// HANDWRITE-END
