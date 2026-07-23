// HANDWRITE-BEGIN gap="missing-generator:logic:workbench-local-diagnostics" tracker="pending-tracker" reason="Retain bounded local startup and sidecar diagnostics without recording terminal input or output."
import Foundation

/// Local diagnostics for recoverable native-host failures.
///
/// The log deliberately excludes terminal bytes and user input. It records only
/// lifecycle, sidecar resolution, request type, and recoverable error context.
public enum WorkbenchDiagnosticLog {
    private static let maximumBytes = 1_048_576
    private static var activeProfile = WorkbenchRuntimeProfile.stable

    public static func configure(profile: WorkbenchRuntimeProfile) {
        activeProfile = profile
    }

    public static func write(_ event: String, details: [String: String] = [:]) {
        guard !Bundle.allBundles.contains(where: { $0.bundlePath.hasSuffix(".xctest") }) else {
            return
        }
        let fileManager = FileManager.default
        let file = activeProfile.logFile(fileManager: fileManager)
        let directory = file.deletingLastPathComponent()
        try? fileManager.createDirectory(at: directory, withIntermediateDirectories: true)

        if let attributes = try? fileManager.attributesOfItem(atPath: file.path),
           let size = attributes[.size] as? NSNumber,
           size.intValue > maximumBytes
        {
            try? fileManager.removeItem(at: file)
        }

        let fields = details
            .sorted { $0.key < $1.key }
            .map { "\($0.key)=\($0.value.replacingOccurrences(of: "\n", with: "\\n"))" }
            .joined(separator: " ")
        let timestamp = ISO8601DateFormatter().string(from: Date())
        let line = "\(timestamp) \(event)\(fields.isEmpty ? "" : " \(fields)")\n"
        guard let data = line.data(using: .utf8) else { return }
        if fileManager.fileExists(atPath: file.path),
           let handle = try? FileHandle(forWritingTo: file)
        {
            defer { try? handle.close() }
            _ = try? handle.seekToEnd()
            try? handle.write(contentsOf: data)
        } else {
            try? data.write(to: file, options: .atomic)
        }
    }
}
// HANDWRITE-END
