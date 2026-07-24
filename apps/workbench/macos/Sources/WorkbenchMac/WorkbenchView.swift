// HANDWRITE-BEGIN gap="missing-generator:logic:6ce34413" tracker="pending-tracker" reason="Render the native folder sidebar, accessible terminal tabs, plus, lifecycle controls, terminal stack, status, and constrained layout."
import AppKit
import SwiftUI
#if canImport(WorkbenchMacCore)
import WorkbenchMacCore
#endif

/// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#logic
struct WorkbenchView: View {
    @ObservedObject var model: WorkbenchModel
    @State private var projectPendingRemoval: RegisteredProject?
    private let runtimeProfile = WorkbenchRuntimeProfile.from()

    var body: some View {
        NavigationSplitView {
            folderSidebar
                .navigationSplitViewColumnWidth(min: 220, ideal: 260, max: 320)
        } detail: {
            HStack(spacing: 0) {
                terminalWorkspace
                if runtimeProfile == .beta {
                    Divider()
                    auxiliaryColumn
                        .frame(minWidth: 240, idealWidth: 280, maxWidth: 320)
                }
            }
            .accessibilityElement(children: .contain)
            .accessibilityIdentifier("workbench.detail")
        }
        .navigationTitle("Workbench")
        .background(Color(nsColor: .windowBackgroundColor))
        // Keep read-only diagnostics, paths, and lifecycle text copyable. Text
        // inside controls still belongs to the control's click action.
        .textSelection(.enabled)
        .confirmationDialog(
            "Remove \(projectPendingRemoval?.displayName ?? "project") from Workbench?",
            isPresented: Binding(
                get: { projectPendingRemoval != nil },
                set: { if !$0 { projectPendingRemoval = nil } }
            ),
            titleVisibility: .visible
        ) {
            Button("Remove Project", role: .destructive) {
                if let project = projectPendingRemoval {
                    model.removeProject(project.id)
                }
                projectPendingRemoval = nil
            }
            Button("Cancel", role: .cancel) {
                projectPendingRemoval = nil
            }
        } message: {
            Text("Only Workbench metadata will be removed. The project files and running terminals stay unchanged.")
        }
    }

    private var folderSidebar: some View {
        VStack(alignment: .leading, spacing: 16) {
            VStack(alignment: .leading, spacing: 4) {
                Text("PROJECTS")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                Text("Project")
                    .font(.title2.weight(.semibold))
            }

            Button(action: chooseFolder) {
                Label("Add Project", systemImage: "folder.badge.plus")
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
            .accessibilityIdentifier("projects.add")
            .accessibilityHint("Registers a project for future terminal launches")

            if model.projects.isEmpty {
                ContentUnavailableView(
                    "No projects",
                    systemImage: "folder",
                    description: Text("Add a project to choose where new terminals start.")
                )
                .frame(maxWidth: .infinity, minHeight: 160)
                .accessibilityIdentifier("projects.empty")
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(model.projects) { project in
                            HStack(spacing: 0) {
                                Button {
                                    model.selectProject(project.id)
                                } label: {
                                    HStack(spacing: 8) {
                                        Image(systemName: model.selectedProjectId == project.id ? "folder.fill" : "folder")
                                            .foregroundStyle(model.selectedProjectId == project.id ? Color.accentColor : Color.secondary)
                                        VStack(alignment: .leading, spacing: 1) {
                                            Text(project.displayName)
                                                .font(.subheadline.weight(model.selectedProjectId == project.id ? .semibold : .regular))
                                                .foregroundStyle(.primary)
                                            Text(project.rootPath)
                                                .font(.caption2)
                                                .foregroundStyle(.secondary)
                                                .lineLimit(1)
                                        }
                                        Spacer(minLength: 0)
                                    }
                                    .padding(.leading, 8)
                                    .padding(.trailing, 4)
                                    .frame(minHeight: 44)
                                    .contentShape(RoundedRectangle(cornerRadius: 6))
                                }
                                .buttonStyle(.plain)
                                .contentShape(RoundedRectangle(cornerRadius: 6))
                                .accessibilityIdentifier("project.\(project.id)")
                                .accessibilityLabel("\(project.displayName), \(project.rootPath)")
                                .accessibilityAddTraits(model.selectedProjectId == project.id ? .isSelected : [])

                                Button {
                                    projectPendingRemoval = project
                                } label: {
                                    Image(systemName: "trash")
                                        .foregroundStyle(.secondary)
                                        .frame(width: 44, height: 44)
                                        .contentShape(Rectangle())
                                }
                                .buttonStyle(.plain)
                                .accessibilityIdentifier("project.remove.\(project.id)")
                                .accessibilityLabel("Remove \(project.displayName)")
                            }
                            .background(
                                model.selectedProjectId == project.id
                                    ? Color.accentColor.opacity(0.12)
                                    : Color.clear,
                                in: RoundedRectangle(cornerRadius: 6)
                            )
                        }
                    }
                }
                .accessibilityIdentifier("projects.list")
            }

        }
        .padding(.horizontal, 18)
        .padding(.bottom, 18)
        .padding(.top, 48)
        .background(.thinMaterial)
    }

    private var terminalWorkspace: some View {
        VStack(spacing: 0) {
            terminalTabStrip
            Divider()
            terminalBody
        }
        .background(Color(nsColor: .windowBackgroundColor))
        .accessibilityIdentifier("terminal.workspace")
    }

    @ViewBuilder
    private var auxiliaryColumn: some View {
        VStack(alignment: .leading, spacing: 12) {
            VStack(alignment: .leading, spacing: 3) {
                Text("AUXILIARY")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.secondary)
                Text("Files")
                    .font(.headline.weight(.semibold))
            }

            if let workspace = model.selectedWorkspace {
                Text(workspace.project.rootPath)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                    .textSelection(.enabled)
                    .accessibilityIdentifier("auxiliary.files.root")
            }

            filesContent
        }
        .padding(14)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(Color(nsColor: .controlBackgroundColor).opacity(0.35))
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("auxiliary.column")
        .accessibilityLabel("Auxiliary Files")
    }

    @ViewBuilder
    private var filesContent: some View {
        switch model.projectFileListing {
        case .noProject:
            ContentUnavailableView(
                "No project selected",
                systemImage: "folder",
                description: Text("Select a project to see its top-level files.")
            )
            .frame(maxWidth: .infinity, minHeight: 160)
            .accessibilityIdentifier("auxiliary.files.no-project")
        case .empty:
            ContentUnavailableView(
                "No visible files",
                systemImage: "folder",
                description: Text("This project root has no visible top-level items.")
            )
            .frame(maxWidth: .infinity, minHeight: 160)
            .accessibilityIdentifier("auxiliary.files.empty")
        case let .unavailable(message):
            ContentUnavailableView(
                "Files unavailable",
                systemImage: "exclamationmark.triangle",
                description: Text(message)
            )
            .frame(maxWidth: .infinity, minHeight: 160)
            .accessibilityIdentifier("auxiliary.files.unavailable")
        case let .available(entries, isTruncated):
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 2) {
                    ForEach(entries) { entry in
                        HStack(spacing: 8) {
                            Image(systemName: entry.kind == .directory ? "folder.fill" : "doc")
                                .foregroundStyle(entry.kind == .directory ? Color.accentColor : Color.secondary)
                                .frame(width: 16)
                                .accessibilityHidden(true)
                            Text(entry.name)
                                .font(.subheadline)
                                .lineLimit(1)
                            Spacer(minLength: 0)
                        }
                        .padding(.horizontal, 6)
                        .frame(minHeight: 28)
                        .contentShape(Rectangle())
                        .accessibilityIdentifier("auxiliary.file.(entry.id)")
                        .accessibilityLabel("\(entry.kind == .directory ? "Folder" : "File") \(entry.name)")
                    }
                    if isTruncated {
                        Text("Showing the first \(ProjectFileListing.maximumEntries) visible items.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .padding(.top, 8)
                            .accessibilityIdentifier("auxiliary.files.truncated")
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .accessibilityIdentifier("auxiliary.files.list")
        }
    }

    private var terminalTabStrip: some View {
        HStack {
            Spacer()
            Menu {
                ForEach(TerminalProfile.allCases, id: \.self) { profile in
                    Button(profile.label) { model.addTerminal(profile: profile) }
                }
            } label: {
                Image(systemName: "plus")
                    .frame(width: 30, height: 30)
            }
            .menuStyle(.borderlessButton)
            .accessibilityIdentifier("terminal.add-profile")
            .accessibilityLabel("Add terminal profile")
        }
        .padding(.horizontal, 8)
        .frame(height: 38)
        .accessibilityIdentifier("terminal.pane-toolbar")
    }

    @ViewBuilder
    private var terminalBody: some View {
        HStack(spacing: 1) {
            ForEach(model.panes) { pane in
                paneContent(pane)
                if pane.id != model.panes.last?.id { Divider() }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    @ViewBuilder
    private func paneContent(_ pane: TerminalPane) -> some View {
        VStack(spacing: 0) {
            HStack(spacing: 7) {
                if let tab = model.tabs.first(where: { $0.id == pane.tabId }) {
                    Circle().fill(stateColor(tab.lifecycle)).frame(width: 6, height: 6)
                    Text(tab.title).font(.caption.weight(.semibold))
                    Spacer()
                    Button {
                        model.selectTab(tab.id)
                        model.splitActivePane()
                    } label: { Image(systemName: "rectangle.split.2x1") }
                        .buttonStyle(.plain).accessibilityLabel("Split right")
                    Button { Task { await model.closeTab(tab.id) } } label: { Image(systemName: "xmark") }
                        .buttonStyle(.plain).accessibilityLabel("Close \(tab.title)")
                } else {
                    Text("New terminal").font(.caption).foregroundStyle(.secondary)
                    Spacer()
                }
            }
            .padding(.horizontal, 8).frame(height: 30)
            Divider()
            if let tab = model.tabs.first(where: { $0.id == pane.tabId }) {
                if tab.lifecycle == .idle { terminalStartState(tab, paneId: pane.id) }
                else if case let .failed(message) = tab.lifecycle { terminalFailureState(tab, message: message, paneId: pane.id) }
                else { terminalSurface(tab, surfaceId: "\(model.selectedProjectId ?? "")::\(pane.id)::\(tab.id)", isActive: model.activePaneId == pane.id) }
            } else {
                ContentUnavailableView("Choose a terminal", systemImage: "terminal")
            }
        }
        .contentShape(Rectangle())
        .onTapGesture { model.selectTab(pane.tabId ?? "") }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var terminalLayers: some View {
        ZStack {
            ForEach(model.mountedTerminalTabs) { scopedTab in
                let tab = scopedTab.tab
                let isActive = scopedTab.projectId == model.selectedProjectId
                    && model.activeTabId == tab.id
                if tab.lifecycle != .idle, !tab.lifecycle.isFailed {
                    terminalSurface(tab, surfaceId: scopedTab.id, isActive: isActive)
                        .opacity(isActive ? 1 : 0)
                        .allowsHitTesting(isActive)
                        .accessibilityHidden(!isActive)
                        .zIndex(isActive ? 1 : 0)
                }
            }
        }
    }

    private func terminalSurface(
        _ tab: TerminalTab,
        surfaceId: String,
        isActive: Bool
    ) -> some View {
        TerminalSurface(
            tabId: surfaceId,
            output: tab.output,
            acceptsInput: tab.lifecycle == .running && isActive,
            onInput: { data in
                Task { await model.sendInput(tabId: tab.id, data: data) }
            },
            onResize: { rows, cols in
                Task { await model.resize(tabId: tab.id, rows: rows, cols: cols) }
            }
        )
        .id(surfaceId)
        .padding(4)
        .background(Color(nsColor: .black))
        // The AppKit terminal itself carries `surfaceId`; keep the public UI
        // query stable for the currently visible logical tab.
        .accessibilityIdentifier("terminal.surface.\(tab.id)")
        .accessibilityLabel("\(tab.title) terminal, \(tab.lifecycle.label)")
    }

    private func terminalStartState(_ tab: TerminalTab, paneId: String) -> some View {
        VStack(spacing: 14) {
            Image(systemName: tab.profile == .shell ? "terminal" : "sparkles")
                .font(.system(size: 32, weight: .medium))
                .foregroundStyle(Color.accentColor)
                .accessibilityHidden(true)
            Text("\(tab.title) is ready")
                .font(.title3.weight(.semibold))
            Text(terminalStartDescription(tab))
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 380)
            if let workspace = model.selectedWorkspace {
                Text(workspace.project.rootPath)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                    .textSelection(.enabled)
                    .frame(maxWidth: 440)
            } else {
                Text("Select a project before starting a terminal.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Button("Start \(tab.title)") {
                model.selectPane(paneId)
                Task { await model.startActiveTab() }
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
            .frame(minHeight: 44)
            .accessibilityIdentifier("terminal.launch.\(tab.id)")
            .accessibilityHint("Starts \(tab.title) in the selected project")
            .disabled(model.selectedFolder == nil)
        }
        .padding(32)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .accessibilityIdentifier("terminal.idle.\(tab.id)")
    }

    private func terminalStartDescription(_ tab: TerminalTab) -> String {
        switch tab.profile {
        case .claude, .codex, .agy:
            "Start a native \(tab.title) session in the selected project."
        case .shell:
            "Use this shell for commands or to start any other installed agent yourself."
        }
    }

    private func terminalFailureState(_ tab: TerminalTab, message: String, paneId: String) -> some View {
        VStack(spacing: 14) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 32, weight: .medium))
                .foregroundStyle(.orange)
                .accessibilityHidden(true)
            Text("\(tab.title) needs attention")
                .font(.title3.weight(.semibold))
            Text(message)
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .textSelection(.enabled)
                .frame(maxWidth: 520)
            Text("Details are recorded in ~/.axiom-workbench/logs/workbench.log")
                .font(.caption)
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
            Button("Copy Details") {
                copyFailureDetails(tab: tab, message: message)
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
            .accessibilityIdentifier("terminal.copy-details.\(tab.id)")
            .accessibilityLabel("Copy \(tab.title) failure details")
            .accessibilityHint("Copies the error message and diagnostic log path")
            Button("Try Again") {
                model.selectPane(paneId)
                Task { await model.startActiveTab() }
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
            .frame(minHeight: 44)
            .accessibilityIdentifier("terminal.retry.\(tab.id)")
            .disabled(model.selectedFolder == nil)
        }
        .padding(32)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .accessibilityIdentifier("terminal.failed.\(tab.id)")
    }

    private func copyFailureDetails(tab: TerminalTab, message: String) {
        let details = """
        \(tab.title) needs attention

        \(message)

        Details are recorded in ~/.axiom-workbench/logs/workbench.log
        """
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(details, forType: .string)
    }

    private func chooseFolder() {
        let panel = NSOpenPanel()
        panel.title = "Add a Workbench project"
        panel.prompt = "Add Project"
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.allowsMultipleSelection = false
        panel.canCreateDirectories = true
        if panel.runModal() == .OK, let url = panel.url {
            model.registerProject(url)
        }
    }

    private func stateColor(_ lifecycle: TerminalLifecycle) -> Color {
        switch lifecycle {
        case .idle: .secondary
        case .starting: .orange
        case .running: .green
        case .exited: .blue
        case .failed: .red
        }
    }
}

// HANDWRITE-END
