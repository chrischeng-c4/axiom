// HANDWRITE-BEGIN gap="missing-generator:logic:cbf6e8bd" tracker="pending-tracker" reason="Mirror the closed Rust wire contract and supervise request-id checked sidecar communication."
import Foundation

public let workbenchCoreProtocolVersion: UInt16 = 1

public enum TerminalProfile: String, Codable, CaseIterable, Sendable {
    case claude
    case codex
    case agy
    case shell

    public var label: String {
        switch self {
        case .claude: "Claude Code"
        case .codex: "Codex"
        case .agy: "AGY"
        case .shell: "Shell"
        }
    }
}

public enum CoreMethod: String, Codable, Sendable {
    case hello
    case launch
    case poll
    case input
    case resize
    case interrupt
    case terminate
    case shutdown
}

public struct CoreParams: Codable, Equatable, Sendable {
    public var tabId: String?
    public var profile: TerminalProfile?
    public var cwd: String?
    public var rows: UInt16?
    public var cols: UInt16?
    public var dataBase64: String?

    public init(
        tabId: String? = nil,
        profile: TerminalProfile? = nil,
        cwd: String? = nil,
        rows: UInt16? = nil,
        cols: UInt16? = nil,
        dataBase64: String? = nil
    ) {
        self.tabId = tabId
        self.profile = profile
        self.cwd = cwd
        self.rows = rows
        self.cols = cols
        self.dataBase64 = dataBase64
    }
}

public struct CoreRequest: Codable, Equatable, Sendable {
    public let protocolVersion: UInt16
    public let requestId: UInt64
    public let method: CoreMethod
    public let params: CoreParams

    public init(requestId: UInt64, method: CoreMethod, params: CoreParams = .init()) {
        protocolVersion = workbenchCoreProtocolVersion
        self.requestId = requestId
        self.method = method
        self.params = params
    }
}

public struct CoreTerminalSnapshot: Codable, Equatable, Sendable {
    public let tabId: String
    public let profile: TerminalProfile
    public let label: String
    public let running: Bool
    public let processId: UInt32?
    public let exitCode: UInt32?
    public let activeCwd: String
    public let cwdSource: String

    public init(
        tabId: String,
        profile: TerminalProfile,
        label: String,
        running: Bool,
        processId: UInt32? = nil,
        exitCode: UInt32? = nil,
        activeCwd: String,
        cwdSource: String = "Launch folder"
    ) {
        self.tabId = tabId
        self.profile = profile
        self.label = label
        self.running = running
        self.processId = processId
        self.exitCode = exitCode
        self.activeCwd = activeCwd
        self.cwdSource = cwdSource
    }
}

public struct CoreTerminalFrame: Codable, Equatable, Sendable {
    public let snapshot: CoreTerminalSnapshot
    public let sequence: UInt64
    public let outputBase64: String

    public init(snapshot: CoreTerminalSnapshot, sequence: UInt64, outputBase64: String) {
        self.snapshot = snapshot
        self.sequence = sequence
        self.outputBase64 = outputBase64
    }
}

public struct CoreResult: Codable, Equatable, Sendable {
    public let kind: String
    public let profiles: [TerminalProfile]?
    public let defaultShell: String?
    public let frame: CoreTerminalFrame?

    public init(
        kind: String,
        profiles: [TerminalProfile]? = nil,
        defaultShell: String? = nil,
        frame: CoreTerminalFrame? = nil
    ) {
        self.kind = kind
        self.profiles = profiles
        self.defaultShell = defaultShell
        self.frame = frame
    }
}

public struct CoreErrorInfo: Codable, Equatable, Sendable {
    public let code: String
    public let message: String

    public init(code: String, message: String) {
        self.code = code
        self.message = message
    }
}

public struct CoreResponse: Codable, Equatable, Sendable {
    public let protocolVersion: UInt16
    public let requestId: UInt64
    public let ok: Bool
    public let result: CoreResult?
    public let error: CoreErrorInfo?

    public init(
        protocolVersion: UInt16 = workbenchCoreProtocolVersion,
        requestId: UInt64,
        ok: Bool,
        result: CoreResult? = nil,
        error: CoreErrorInfo? = nil
    ) {
        self.protocolVersion = protocolVersion
        self.requestId = requestId
        self.ok = ok
        self.result = result
        self.error = error
    }
}

public enum CoreClientError: LocalizedError, Equatable {
    case executableUnavailable(String)
    case sidecarClosed
    case protocolMismatch(UInt16)
    case responseMismatch(expected: UInt64, actual: UInt64)
    case remote(CoreErrorInfo)
    case transport(String)

    public var errorDescription: String? {
        switch self {
        case let .executableUnavailable(path):
            "workbench-core is unavailable at \(path). Build it with cargo build -p workbench --bin workbench-core."
        case .sidecarClosed:
            "workbench-core closed before responding. You can start the terminal again."
        case let .protocolMismatch(version):
            "workbench-core protocol \(version) does not match client protocol \(workbenchCoreProtocolVersion)."
        case let .responseMismatch(expected, actual):
            "workbench-core response id \(actual) did not match request id \(expected)."
        case let .remote(error):
            error.message
        case let .transport(message):
            message
        }
    }
}

public protocol CoreClientProtocol: AnyObject {
    func send(method: CoreMethod, params: CoreParams) async throws -> CoreResponse
    func shutdown() async
}

/// One serialized request stream to the Rust PTY sidecar.
///
/// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#logic
public actor RustCoreClient: CoreClientProtocol {
    private let explicitExecutableURL: URL?
    private var process: Process?
    private var inputHandle: FileHandle?
    private var outputHandle: FileHandle?
    private var outputBuffer = Data()
    private var nextRequestId: UInt64 = 1

    public init(executableURL: URL? = nil) {
        explicitExecutableURL = executableURL
    }

    public func send(method: CoreMethod, params: CoreParams = .init()) async throws -> CoreResponse {
        try ensureProcess()
        let requestId = nextRequestId
        nextRequestId += 1
        let request = CoreRequest(requestId: requestId, method: method, params: params)
        var payload = try JSONEncoder().encode(request)
        payload.append(0x0A)
        do {
            try inputHandle?.write(contentsOf: payload)
            let response = try JSONDecoder().decode(CoreResponse.self, from: readResponseLine())
            guard response.protocolVersion == workbenchCoreProtocolVersion else {
                throw CoreClientError.protocolMismatch(response.protocolVersion)
            }
            guard response.requestId == requestId else {
                throw CoreClientError.responseMismatch(expected: requestId, actual: response.requestId)
            }
            if let error = response.error {
                throw CoreClientError.remote(error)
            }
            return response
        } catch let error as CoreClientError {
            WorkbenchDiagnosticLog.write("sidecar.request_failed", details: [
                "method": method.rawValue,
                "tab": params.tabId ?? "",
                "error": error.localizedDescription,
            ])
            throw error
        } catch {
            let transportError = CoreClientError.transport(error.localizedDescription)
            WorkbenchDiagnosticLog.write("sidecar.request_failed", details: [
                "method": method.rawValue,
                "tab": params.tabId ?? "",
                "error": transportError.localizedDescription,
            ])
            throw transportError
        }
    }

    public func shutdown() async {
        if process?.isRunning == true {
            _ = try? await send(method: .shutdown, params: .init())
            process?.waitUntilExit()
        }
        inputHandle?.closeFile()
        outputHandle?.closeFile()
        process = nil
        inputHandle = nil
        outputHandle = nil
        outputBuffer.removeAll(keepingCapacity: false)
    }

    private func ensureProcess() throws {
        if process?.isRunning == true {
            return
        }
        let executableURL = try resolveExecutableURL()
        let inputPipe = Pipe()
        let outputPipe = Pipe()
        let process = Process()
        process.executableURL = executableURL
        process.standardInput = inputPipe
        process.standardOutput = outputPipe
        process.standardError = FileHandle.standardError
        do {
            try process.run()
        } catch {
            WorkbenchDiagnosticLog.write("sidecar.start_failed", details: [
                "executable": executableURL.path,
                "error": error.localizedDescription,
            ])
            throw CoreClientError.transport("Unable to start workbench-core: \(error.localizedDescription)")
        }
        WorkbenchDiagnosticLog.write("sidecar.started", details: ["executable": executableURL.path])
        self.process = process
        inputHandle = inputPipe.fileHandleForWriting
        outputHandle = outputPipe.fileHandleForReading
        outputBuffer.removeAll(keepingCapacity: false)
    }

    private func resolveExecutableURL() throws -> URL {
        let fileManager = FileManager.default
        var candidates: [URL] = []
        if let explicitExecutableURL {
            candidates.append(explicitExecutableURL)
        }
        if let configured = ProcessInfo.processInfo.environment["WORKBENCH_CORE_BIN"] {
            candidates.append(URL(fileURLWithPath: configured))
        }
        if let appExecutable = Bundle.main.executableURL {
            candidates.append(appExecutable.deletingLastPathComponent().appendingPathComponent("workbench-core"))
            candidates.append(
                appExecutable
                    .deletingLastPathComponent()
                    .deletingLastPathComponent()
                    .appendingPathComponent("Resources/workbench-core")
            )
        }
        candidates.append(
            URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
                .appendingPathComponent("target/debug/workbench-core")
        )
        // SwiftPM launches the executable through Launch Services, which does
        // not preserve the package checkout as the current directory. Keep a
        // development-only source-root candidate after normal bundle paths.
        candidates.append(
            URL(fileURLWithPath: #filePath)
                .deletingLastPathComponent()
                .deletingLastPathComponent()
                .deletingLastPathComponent()
                .deletingLastPathComponent()
                .deletingLastPathComponent()
                .appendingPathComponent("target/debug/workbench-core")
        )
        if let candidate = candidates.first(where: { fileManager.isExecutableFile(atPath: $0.path) }) {
            return candidate
        }
        WorkbenchDiagnosticLog.write("sidecar.executable_missing", details: [
            "candidates": candidates.map(\.path).joined(separator: " | "),
        ])
        throw CoreClientError.executableUnavailable(candidates.first?.path ?? "workbench-core")
    }

    private func readResponseLine() throws -> Data {
        while true {
            if let newline = outputBuffer.firstIndex(of: 0x0A) {
                let line = outputBuffer[..<newline]
                outputBuffer.removeSubrange(...newline)
                return Data(line)
            }
            guard let chunk = outputHandle?.availableData, !chunk.isEmpty else {
                throw CoreClientError.sidecarClosed
            }
            outputBuffer.append(chunk)
        }
    }
}
// HANDWRITE-END
