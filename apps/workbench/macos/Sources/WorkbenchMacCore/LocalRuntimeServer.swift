// HANDWRITE-BEGIN gap="missing-generator:contract:70b19e58" tracker="pending-tracker" reason="Implement the owner-only registry, singleton lock, loopback protocol version, request-id/token checks, uiState activation probe, and snapshot response contract."
import AppKit
import Darwin
import Foundation
import Network

/// The native app's one-user, read-only observability endpoint.
/// It is deliberately not a terminal, project, or agent-control channel.
@MainActor
public final class LocalRuntimeServer {
    private static let protocolVersion = 1
    private static let maximumPNGBytes = 16 * 1024 * 1024
    private let fileManager: FileManager
    private let runtimeDirectory: URL
    private var listener: NWListener?
    private var lockURL: URL?
    private var instanceId = UUID().uuidString
    private var token = UUID().uuidString.replacingOccurrences(of: "-", with: "")

    public init(fileManager: FileManager = .default, runtimeDirectory: URL? = nil) {
        self.fileManager = fileManager
        self.runtimeDirectory = runtimeDirectory ?? fileManager.homeDirectoryForCurrentUser
            .appendingPathComponent(".axiom-workbench/runtime", isDirectory: true)
    }

    public func start() throws {
        guard listener == nil else { return }
        try fileManager.createDirectory(at: runtimeDirectory, withIntermediateDirectories: true)
        try fileManager.setAttributes([.posixPermissions: 0o700], ofItemAtPath: runtimeDirectory.path)
        try acquireLease()

        let listener = try NWListener(using: .tcp, on: .any)
        listener.newConnectionHandler = { [weak self] connection in
            connection.start(queue: .global(qos: .userInitiated))
            Task { @MainActor in
                self?.receiveOneRequest(connection)
            }
        }
        listener.stateUpdateHandler = { [weak self] state in
            guard case .ready = state else { return }
            Task { @MainActor in
                guard let self, self.listener === listener else { return }
                self.publishRegistry(port: listener.port?.rawValue ?? 0)
            }
        }
        listener.start(queue: .global(qos: .userInitiated))
        self.listener = listener
    }

    public func stop() {
        listener?.cancel()
        listener = nil
        try? fileManager.removeItem(at: registryURL)
        if let lockURL { try? fileManager.removeItem(at: lockURL) }
        lockURL = nil
    }

    private var registryURL: URL { runtimeDirectory.appendingPathComponent("current.json") }

    private func acquireLease() throws {
        let candidate = runtimeDirectory.appendingPathComponent("workbench.lock")
        if fileManager.fileExists(atPath: candidate.path) {
            let existingPID = (try? String(contentsOf: candidate, encoding: .utf8)).flatMap(Int32.init)
            if let existingPID, kill(existingPID, 0) == 0 {
                throw LocalRuntimeError.alreadyRunning
            }
            try? fileManager.removeItem(at: candidate)
            try? fileManager.removeItem(at: registryURL)
        }
        let descriptor = open(candidate.path, O_WRONLY | O_CREAT | O_EXCL, S_IRUSR | S_IWUSR)
        guard descriptor >= 0 else {
            throw LocalRuntimeError.cannotAcquireLease
        }
        defer { close(descriptor) }
        let pid = "\(ProcessInfo.processInfo.processIdentifier)".data(using: .utf8) ?? Data()
        guard pid.withUnsafeBytes({ write(descriptor, $0.baseAddress, $0.count) }) == pid.count else {
            try? fileManager.removeItem(at: candidate)
            throw LocalRuntimeError.cannotAcquireLease
        }
        lockURL = candidate
    }

    private func publishRegistry(port: UInt16) {
        guard port >= 1024 else { return }
        let registry: [String: Any] = [
            "protocolVersion": Self.protocolVersion,
            "instanceId": instanceId,
            "pid": ProcessInfo.processInfo.processIdentifier,
            "port": port,
            "token": token,
        ]
        guard let data = try? JSONSerialization.data(withJSONObject: registry, options: [.sortedKeys]) else { return }
        let temporary = runtimeDirectory.appendingPathComponent("current-\(UUID().uuidString).json")
        do {
            try data.write(to: temporary, options: .atomic)
            try fileManager.setAttributes([.posixPermissions: 0o600], ofItemAtPath: temporary.path)
            _ = try fileManager.replaceItemAt(registryURL, withItemAt: temporary)
        } catch {
            try? data.write(to: registryURL, options: .atomic)
            try? fileManager.setAttributes([.posixPermissions: 0o600], ofItemAtPath: registryURL.path)
        }
    }

    private func receiveOneRequest(_ connection: NWConnection) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65_536) { [weak self] data, _, _, _ in
            Task { @MainActor in
                guard let self else { return }
                let response = self.handleRequest(data ?? Data())
                connection.send(content: response, completion: .contentProcessed { _ in connection.cancel() })
            }
        }
    }

    private func handleRequest(_ data: Data) -> Data {
        guard data.count <= 65_536,
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              object["protocolVersion"] as? Int == Self.protocolVersion,
              let requestID = object["requestId"] as? UInt64, requestID != 0,
              let requestToken = object["token"] as? String,
              requestToken == token,
              let method = object["method"] as? String
        else { return encodeError(requestID: 0, message: "invalid or unauthenticated request") }

        switch method {
        case "uiState":
            NSApp.activate(ignoringOtherApps: true)
            return encode(["ok": true, "requestId": requestID])
        case "snapshot":
            do {
                let png = try capturePNG()
                return encode([
                    "ok": true,
                    "requestId": requestID,
                    "mimeType": "image/png",
                    "dataBase64": png.base64EncodedString(),
                ])
            } catch {
                return encodeError(requestID: requestID, message: "content view capture failed")
            }
        default:
            return encodeError(requestID: requestID, message: "unsupported method")
        }
    }

    private func encodeError(requestID: UInt64, message: String) -> Data {
        encode(["ok": false, "requestId": requestID, "error": message])
    }

    private func encode(_ fields: [String: Any]) -> Data {
        var envelope = fields
        envelope["protocolVersion"] = Self.protocolVersion
        envelope["instanceId"] = instanceId
        let payload = (try? JSONSerialization.data(withJSONObject: envelope, options: [.sortedKeys])) ?? Data("{}".utf8)
        return payload + Data([0x0A])
    }

    public func capturePNG() throws -> Data {
        guard let view = NSApp.keyWindow?.contentView ?? NSApp.windows.first(where: { $0.isVisible })?.contentView else {
            throw LocalRuntimeError.noContentView
        }
        return try capturePNG(from: view)
    }

    public func capturePNG(from view: NSView) throws -> Data {
        let bounds = view.bounds.integral
        guard bounds.width > 0, bounds.height > 0,
              let representation = view.bitmapImageRepForCachingDisplay(in: bounds)
        else { throw LocalRuntimeError.noContentView }
        view.cacheDisplay(in: bounds, to: representation)
        guard let png = representation.representation(using: .png, properties: [:]), png.count <= Self.maximumPNGBytes else {
            throw LocalRuntimeError.pngTooLarge
        }
        return png
    }
}

public enum LocalRuntimeError: Error {
    case alreadyRunning
    case cannotAcquireLease
    case noContentView
    case pngTooLarge
}
// HANDWRITE-END
