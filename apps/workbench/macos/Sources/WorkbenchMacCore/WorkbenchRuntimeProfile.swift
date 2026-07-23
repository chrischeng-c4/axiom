// HANDWRITE-BEGIN gap="missing-generator:logic:workbench-runtime-profile" tracker="#2445" reason="Define the closed native channel identity and its isolated state root."
import Foundation

/// The app bundle supplies this identity; runtime state never guesses it from
/// the process name or another profile's registry.
public enum WorkbenchRuntimeProfile: String, CaseIterable, Sendable {
    case stable
    case beta

    public static func from(bundle: Bundle = .main) -> WorkbenchRuntimeProfile {
        guard let value = bundle.object(forInfoDictionaryKey: "WORKBENCH_RUNTIME_PROFILE") as? String,
              let profile = WorkbenchRuntimeProfile(rawValue: value)
        else {
            return .stable
        }
        return profile
    }

    public var productName: String {
        switch self {
        case .stable: "Axiom Workbench"
        case .beta: "Axiom Workbench Beta"
        }
    }

    public var bundleIdentifier: String {
        switch self {
        case .stable: "com.axiom.workbench"
        case .beta: "com.axiom.workbench.beta"
        }
    }

    public func stateRoot(fileManager: FileManager = .default) -> URL {
        let name = switch self {
        case .stable: ".axiom-workbench"
        case .beta: ".axiom-workbench-beta"
        }
        return fileManager.homeDirectoryForCurrentUser.appendingPathComponent(name, isDirectory: true)
    }

    public func projectsRoot(fileManager: FileManager = .default) -> URL {
        stateRoot(fileManager: fileManager)
    }

    public func logFile(fileManager: FileManager = .default) -> URL {
        stateRoot(fileManager: fileManager)
            .appendingPathComponent("logs", isDirectory: true)
            .appendingPathComponent("workbench.log")
    }

    public func runtimeRoot(fileManager: FileManager = .default) -> URL {
        stateRoot(fileManager: fileManager).appendingPathComponent("runtime", isDirectory: true)
    }
}
// HANDWRITE-END
