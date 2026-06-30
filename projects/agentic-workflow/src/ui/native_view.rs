#![allow(unexpected_cfgs)]

use anyhow::Result;

use crate::cli::view::{RepoViewItemSnapshot, RepoViewSnapshot};

pub fn catalog_text(snapshot: &RepoViewSnapshot) -> String {
    let selected = snapshot.selected_repo.as_deref().unwrap_or_default();
    let mut out = String::new();
    out.push_str(&format!(
        "AW Repo View\n{}\n\n{} repos / {} projects / {} libs\n\n",
        snapshot.repo.name,
        snapshot.repo_catalog.len(),
        snapshot.repo.project_count,
        snapshot.repo.library_count
    ));
    for item in &snapshot.repo_catalog {
        let marker = if item.path == selected { ">" } else { " " };
        out.push_str(&format!(
            "{marker} {}\n  {}\n  {} items / {} projects / {} libs\n",
            item.name, item.path, item.item_count, item.project_count, item.library_count
        ));
    }
    out
}

pub fn detail_text(snapshot: &RepoViewSnapshot) -> String {
    let Some(item) = selected_item(snapshot) else {
        return "No repo item selected.".to_string();
    };
    let mut out = String::new();
    out.push_str(&format!(
        "{} [{}]\n{}\n\nProject / lib selector\n",
        item.readme.title, item.project.kind, item.project.path
    ));
    for catalog_item in &snapshot.catalog {
        let marker = if catalog_item.name == item.project.name {
            ">"
        } else {
            " "
        };
        out.push_str(&format!(
            "{marker} {} [{}]\n",
            catalog_item.name, catalog_item.kind
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "Summary\nCapabilities: {}\nEC cases: {}\nTD markdown files: {}\nTD capability refs: {}\n\n",
        item.capabilities.count,
        item.ec.case_count,
        item.td.markdown_file_count,
        item.td.capability_ref_count
    ));
    out.push_str("Capabilities\n");
    out.push_str("Title | Status | Type | Surface | EC | TD\n");
    out.push_str("----- | ------ | ---- | ------- | -- | --\n");
    for cap in &item.capabilities.items {
        out.push_str(&format!(
            "{} | {} | {} | {} | {} | {}\n",
            cap.title,
            cap.status,
            cap.capability_type.as_deref().unwrap_or("-"),
            cap.surface_count,
            cap.ec_case_count,
            cap.td_ref_count
        ));
    }
    out.push_str("\nEC Cases\n");
    if item.ec.cases.is_empty() {
        out.push_str("- none\n");
    } else {
        for case in &item.ec.cases {
            out.push_str(&format!(
                "- {} [{}] {}\n  {}\n",
                case.id, case.category, case.capability_id, case.command
            ));
        }
    }
    out.push_str(&format!(
        "\nREADME\n{}\n\n{}\n\nTD\n{} markdown files / {} capability refs\n{}\n",
        item.readme.path,
        item.readme.brief,
        item.td.markdown_file_count,
        item.td.capability_ref_count,
        item.td.root
    ));
    if !snapshot.warnings.is_empty() || !item.warnings.is_empty() {
        out.push_str("\nWarnings\n");
        for warning in &snapshot.warnings {
            out.push_str(&format!("- {warning}\n"));
        }
        for warning in &item.warnings {
            out.push_str(&format!("- {warning}\n"));
        }
    }
    out
}

fn selected_item(snapshot: &RepoViewSnapshot) -> Option<&RepoViewItemSnapshot> {
    snapshot
        .selected
        .as_deref()
        .and_then(|selected| {
            snapshot
                .items
                .iter()
                .find(|item| item.project.name == selected)
        })
        .or_else(|| snapshot.items.first())
}

#[cfg(target_os = "macos")]
pub fn run_native_repo_view(snapshot: &RepoViewSnapshot) -> Result<()> {
    macos::run(snapshot)
}

#[cfg(not(target_os = "macos"))]
pub fn run_native_repo_view(_snapshot: &RepoViewSnapshot) -> Result<()> {
    anyhow::bail!("aw view native desktop mode currently supports macOS only")
}

#[cfg(target_os = "macos")]
mod macos {
    use super::{catalog_text, detail_text};
    use crate::cli::view::{
        layout_toggle_button_label, toggled_view_layout, RepoViewSnapshot, APP_SCREENSHOT_HEIGHT,
        APP_SCREENSHOT_WIDTH,
    };
    use anyhow::Result;
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel, NO, YES};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::{c_void, CStr, CString};
    use std::fs::File;
    use std::io::{Read, Write};
    use std::os::raw::c_char;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::FromRawFd;
    use std::path::PathBuf;
    use std::ptr;
    use std::sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex, Once,
    };
    use std::thread;

    #[link(name = "AppKit", kind = "framework")]
    extern "C" {}

    type Id = *mut Object;

    const NS_UTF8_STRING_ENCODING: usize = 4;
    const NS_BACKING_STORE_BUFFERED: usize = 2;
    const NS_WINDOW_STYLE_MASK_TITLED: usize = 1 << 0;
    const NS_WINDOW_STYLE_MASK_CLOSABLE: usize = 1 << 1;
    const NS_WINDOW_STYLE_MASK_MINIATURIZABLE: usize = 1 << 2;
    const NS_WINDOW_STYLE_MASK_RESIZABLE: usize = 1 << 3;
    const NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE: usize = 1 << 1;
    const NS_VIEW_WIDTH_SIZABLE: usize = 1 << 1;
    const NS_VIEW_HEIGHT_SIZABLE: usize = 1 << 4;
    const NS_VIEW_MIN_X_MARGIN: usize = 1 << 0;
    const NS_VIEW_MIN_Y_MARGIN: usize = 1 << 3;
    const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: isize = 0;
    const NS_BEZEL_STYLE_ROUNDED: usize = 1;
    const NS_BOX_CUSTOM: usize = 4;
    const NS_NO_TITLE: usize = 0;
    const NS_LINE_BORDER: usize = 1;

    struct NativeRepoViewState {
        snapshot: RepoViewSnapshot,
        root_view: Id,
        root_background_view: Id,
        header_view: Id,
        header_rule_view: Id,
        header_title_label: Id,
        header_summary_label: Id,
        catalog_panel_view: Id,
        catalog_header_view: Id,
        catalog_title_label: Id,
        catalog_count_label: Id,
        button: Id,
        controller: Id,
        terminal_panel_view: Id,
        terminal_header_view: Id,
        terminal_title_label: Id,
        terminal_badge_label: Id,
        detail_panel_view: Id,
        detail_header_label: Id,
        project_selector: Id,
        catalog_scroll_view: Id,
        catalog_text_view: Id,
        detail_scroll_view: Id,
        detail_text_view: Id,
        terminal_scroll_view: Id,
        terminal_output_view: Id,
        terminal_input: Id,
        terminal_tx: Sender<TerminalEvent>,
        terminal_rx: Receiver<TerminalEvent>,
        terminal_session: Option<TerminalPtySession>,
        terminal_cwd: PathBuf,
        terminal_log: String,
        render_scheduler: RenderScheduler,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum RedrawReason {
        LayoutToggle,
        ProjectSelection,
        WindowResize,
        TerminalInput,
        TerminalOutput,
    }

    #[derive(Debug, Clone, Copy)]
    struct FrameClock {
        frames_per_second: f64,
    }

    impl FrameClock {
        fn from_main_screen() -> Self {
            Self {
                frames_per_second: unsafe { main_screen_maximum_frames_per_second() },
            }
        }

        fn coalescing_interval(self) -> f64 {
            (1.0 / self.frames_per_second).clamp(1.0 / 240.0, 1.0 / 30.0)
        }
    }

    #[derive(Debug)]
    struct RenderScheduler {
        scheduled: bool,
        pending: Vec<RedrawReason>,
        frame_clock: FrameClock,
        frame_count: u64,
    }

    #[derive(Debug, Clone, Copy)]
    struct RenderFrame {
        redraw_terminal: bool,
        redraw_controls: bool,
    }

    impl RenderScheduler {
        fn new(frame_clock: FrameClock) -> Self {
            Self {
                scheduled: false,
                pending: Vec::new(),
                frame_clock,
                frame_count: 0,
            }
        }

        fn request_redraw(&mut self, reason: RedrawReason) -> bool {
            if !self.pending.contains(&reason) {
                self.pending.push(reason);
            }
            if self.scheduled {
                return false;
            }
            self.scheduled = true;
            true
        }

        fn take_frame(&mut self) -> RenderFrame {
            let pending = std::mem::take(&mut self.pending);
            self.scheduled = false;
            self.frame_count += 1;
            let layout_dirty = pending.contains(&RedrawReason::LayoutToggle);
            let project_dirty = pending.contains(&RedrawReason::ProjectSelection);
            let resize_dirty = pending.contains(&RedrawReason::WindowResize);
            let terminal_dirty = pending.iter().any(|reason| {
                matches!(
                    reason,
                    RedrawReason::TerminalInput | RedrawReason::TerminalOutput
                )
            });
            RenderFrame {
                redraw_terminal: terminal_dirty,
                redraw_controls: layout_dirty || project_dirty || resize_dirty,
            }
        }

        fn coalescing_interval(&self) -> f64 {
            self.frame_clock.coalescing_interval()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TerminalStream {
        Pty,
        System,
    }

    #[derive(Debug)]
    enum TerminalEvent {
        Output {
            stream: TerminalStream,
            text: String,
        },
        Exit {
            code: Option<i32>,
        },
    }

    struct TerminalPtySession {
        writer: Arc<Mutex<File>>,
        child_pid: libc::pid_t,
    }

    impl TerminalPtySession {
        fn write_command(&self, command: &str) -> std::io::Result<()> {
            let mut writer = self
                .writer
                .lock()
                .map_err(|_| std::io::Error::other("terminal PTY writer lock poisoned"))?;
            writer.write_all(command.as_bytes())?;
            writer.write_all(b"\n")?;
            writer.flush()
        }
    }

    impl Drop for TerminalPtySession {
        fn drop(&mut self) {
            unsafe {
                let _ = libc::kill(self.child_pid, libc::SIGHUP);
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSSize {
        width: f64,
        height: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSRect {
        origin: NSPoint,
        size: NSSize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSRange {
        location: usize,
        length: usize,
    }

    #[derive(Clone, Copy, Debug)]
    struct NativeViewGeometry {
        width: f64,
        height: f64,
        gap: f64,
        content_top: f64,
        catalog_x: f64,
        catalog_width: f64,
        right_x: f64,
        right_width: f64,
        content_height: f64,
    }

    impl NativeViewGeometry {
        fn initial() -> Self {
            Self::from_size(size(
                APP_SCREENSHOT_WIDTH as f64,
                APP_SCREENSHOT_HEIGHT as f64,
            ))
        }

        fn from_size(size: NSSize) -> Self {
            let width = size.width.max(980.0);
            let height = size.height.max(620.0);
            let margin = 24.0;
            let gap = 16.0;
            let content_top = 82.0;
            let content_bottom = 30.0;
            let catalog_x = margin;
            let catalog_width = (width * 0.20).clamp(250.0, 320.0);
            let right_x = catalog_x + catalog_width + gap;
            let right_width = (width - right_x - margin).max(620.0);
            let content_height = (height - content_top - content_bottom).max(480.0);
            Self {
                width,
                height,
                gap,
                content_top,
                catalog_x,
                catalog_width,
                right_x,
                right_width,
                content_height,
            }
        }

        fn root_frame(self) -> NSRect {
            rect(0.0, 0.0, self.width, self.height)
        }

        fn rect_from_top_left(self, x: f64, y: f64, width: f64, height: f64) -> NSRect {
            rect(x, self.height - y - height, width.max(1.0), height.max(1.0))
        }
    }

    pub fn run(snapshot: &RepoViewSnapshot) -> Result<()> {
        unsafe {
            let app: Id = msg_send![class!(NSApplication), sharedApplication];
            let _: () =
                msg_send![app, setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
            let _: () = msg_send![app, finishLaunching];

            let window = make_window(snapshot)?;
            let _: () = msg_send![window, setReleasedWhenClosed: NO];
            let _: () = msg_send![window, display];
            let _: () = msg_send![window, makeKeyAndOrderFront: ptr::null_mut::<Object>()];
            let _: () = msg_send![window, orderFrontRegardless];
            let _: () = msg_send![app, run];
        }
        Ok(())
    }

    unsafe fn make_window(snapshot: &RepoViewSnapshot) -> Result<Id> {
        let frame = rect(
            0.0,
            0.0,
            APP_SCREENSHOT_WIDTH as f64,
            APP_SCREENSHOT_HEIGHT as f64,
        );
        let style = NS_WINDOW_STYLE_MASK_TITLED
            | NS_WINDOW_STYLE_MASK_CLOSABLE
            | NS_WINDOW_STYLE_MASK_MINIATURIZABLE
            | NS_WINDOW_STYLE_MASK_RESIZABLE;
        let window: Id = msg_send![class!(NSWindow), alloc];
        let window: Id = msg_send![
            window,
            initWithContentRect: frame
            styleMask: style
            backing: NS_BACKING_STORE_BUFFERED
            defer: NO
        ];
        let _: () = msg_send![
            window,
            setCollectionBehavior: NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE
        ];
        let _: () = msg_send![window, setMinSize: size(980.0, 620.0)];
        let title = ns_string(&format!("AW Repo View - {}", snapshot.repo.name));
        let _: () = msg_send![window, setTitle: title];
        let _: () = msg_send![window, center];

        let (content_view, terminal_input) = make_content_view(snapshot, window)?;
        let _: () = msg_send![window, setContentView: content_view];
        let _: bool = msg_send![window, makeFirstResponder: terminal_input];
        Ok(window)
    }

    unsafe fn make_content_view(snapshot: &RepoViewSnapshot, window: Id) -> Result<(Id, Id)> {
        let frame = rect(
            0.0,
            0.0,
            APP_SCREENSHOT_WIDTH as f64,
            APP_SCREENSHOT_HEIGHT as f64,
        );
        let geometry = NativeViewGeometry::initial();
        let root_view: Id = msg_send![class!(NSView), alloc];
        let root_view: Id = msg_send![root_view, initWithFrame: frame];
        let _: () = msg_send![root_view, setAutoresizingMask: autoresize_mask()];

        let root_background = make_box(
            geometry.root_frame(),
            terminal_color(244, 246, 248),
            terminal_color(244, 246, 248),
            0.0,
            0.0,
        );
        let header_view = make_box(
            geometry.rect_from_top_left(0.0, 0.0, geometry.width, 58.0),
            terminal_color(31, 42, 55),
            terminal_color(31, 42, 55),
            0.0,
            0.0,
        );
        let header_rule = make_box(
            geometry.rect_from_top_left(0.0, 58.0, geometry.width, 1.0),
            terminal_color(96, 165, 250),
            terminal_color(96, 165, 250),
            0.0,
            0.0,
        );
        let header_title = make_label(
            geometry.rect_from_top_left(28.0, 17.0, 170.0, 28.0),
            "AW Repo View",
            22.0,
            terminal_color(248, 250, 252),
        );
        let header_summary = make_label(
            geometry.rect_from_top_left(210.0, 20.0, geometry.width - 460.0, 22.0),
            &format!(
                "{} - {} repos / {} projects / {} libs",
                snapshot.repo.name,
                snapshot.repo_catalog.len(),
                snapshot.repo.project_count,
                snapshot.repo.library_count
            ),
            14.0,
            terminal_color(203, 213, 225),
        );
        let catalog_chrome = make_catalog_chrome(snapshot, geometry);
        let terminal_chrome = make_terminal_chrome(snapshot, geometry);
        let detail_chrome = make_detail_chrome(snapshot, geometry);
        let project_selector = make_project_selector(snapshot, geometry);
        let (catalog_scroll_view, catalog_text_view) = make_catalog_views(geometry);
        let (detail_scroll_view, detail_text_view) = make_detail_views(snapshot, geometry);
        let (terminal_scroll_view, terminal_output_view, terminal_input) =
            make_terminal_views(snapshot, geometry);
        let button = make_layout_toggle_button(snapshot, geometry);
        let (terminal_tx, terminal_rx) = mpsc::channel();
        let state = Box::into_raw(Box::new(NativeRepoViewState {
            snapshot: snapshot.clone(),
            root_view,
            root_background_view: root_background,
            header_view,
            header_rule_view: header_rule,
            header_title_label: header_title,
            header_summary_label: header_summary,
            catalog_panel_view: catalog_chrome.panel,
            catalog_header_view: catalog_chrome.header,
            catalog_title_label: catalog_chrome.title,
            catalog_count_label: catalog_chrome.count,
            button,
            controller: ptr::null_mut(),
            terminal_panel_view: terminal_chrome.panel,
            terminal_header_view: terminal_chrome.header,
            terminal_title_label: terminal_chrome.title,
            terminal_badge_label: terminal_chrome.badge,
            detail_panel_view: detail_chrome.panel,
            detail_header_label: detail_chrome.header_label,
            project_selector,
            catalog_scroll_view,
            catalog_text_view,
            detail_scroll_view,
            detail_text_view,
            terminal_scroll_view,
            terminal_output_view,
            terminal_input,
            terminal_tx,
            terminal_rx,
            terminal_session: None,
            terminal_cwd: PathBuf::from(&snapshot.repo.root),
            terminal_log: initial_terminal_log(snapshot),
            render_scheduler: RenderScheduler::new(FrameClock::from_main_screen()),
        }));
        let controller = make_layout_toggle_controller(state.cast::<c_void>());
        let _: Id = msg_send![controller, retain];
        (*state).controller = controller;
        let _: () = msg_send![button, setTarget: controller];
        let _: () = msg_send![button, setAction: sel!(toggleLayout:)];
        let _: () = msg_send![project_selector, setTarget: controller];
        let _: () = msg_send![project_selector, setAction: sel!(selectProject:)];
        let _: () = msg_send![terminal_input, setTarget: controller];
        let _: () = msg_send![terminal_input, setAction: sel!(runTerminalCommand:)];
        observe_window_resize(controller, window);

        let state_ref = &mut *(state.cast::<NativeRepoViewState>());
        state_ref.terminal_session = start_terminal_pty_session(
            state_ref.terminal_cwd.clone(),
            state_ref.terminal_tx.clone(),
            controller,
        )
        .map_err(|err| {
            state_ref
                .terminal_log
                .push_str(&format!("terminal PTY unavailable: {err}\n"));
            err
        })
        .ok();
        set_catalog_output(state_ref);
        set_detail_output(state_ref);
        set_terminal_output(state_ref);

        let _: () = msg_send![root_view, addSubview: root_background];
        let _: () = msg_send![root_view, addSubview: header_view];
        let _: () = msg_send![root_view, addSubview: header_rule];
        let _: () = msg_send![root_view, addSubview: header_title];
        let _: () = msg_send![root_view, addSubview: header_summary];
        let _: () = msg_send![root_view, addSubview: catalog_chrome.panel];
        let _: () = msg_send![root_view, addSubview: catalog_chrome.header];
        let _: () = msg_send![root_view, addSubview: catalog_chrome.title];
        let _: () = msg_send![root_view, addSubview: catalog_chrome.count];
        let _: () = msg_send![root_view, addSubview: terminal_chrome.panel];
        let _: () = msg_send![root_view, addSubview: terminal_chrome.header];
        let _: () = msg_send![root_view, addSubview: terminal_chrome.title];
        let _: () = msg_send![root_view, addSubview: terminal_chrome.badge];
        let _: () = msg_send![root_view, addSubview: detail_chrome.panel];
        let _: () = msg_send![root_view, addSubview: detail_chrome.header_label];
        let _: () = msg_send![root_view, addSubview: project_selector];
        let _: () = msg_send![root_view, addSubview: catalog_scroll_view];
        let _: () = msg_send![root_view, addSubview: detail_scroll_view];
        let _: () = msg_send![root_view, addSubview: terminal_scroll_view];
        let _: () = msg_send![root_view, addSubview: terminal_input];
        let _: () = msg_send![root_view, addSubview: button];
        Ok((root_view, terminal_input))
    }

    struct CatalogChrome {
        panel: Id,
        header: Id,
        title: Id,
        count: Id,
    }

    #[derive(Clone, Copy)]
    struct TerminalChrome {
        panel: Id,
        header: Id,
        title: Id,
        badge: Id,
    }

    #[derive(Clone, Copy)]
    struct DetailChrome {
        panel: Id,
        header_label: Id,
    }

    unsafe fn make_catalog_chrome(
        snapshot: &RepoViewSnapshot,
        geometry: NativeViewGeometry,
    ) -> CatalogChrome {
        let panel = make_box(
            catalog_panel_frame(geometry),
            terminal_color(255, 255, 255),
            terminal_color(203, 213, 225),
            1.0,
            0.0,
        );
        let header = make_box(
            geometry.rect_from_top_left(
                geometry.catalog_x,
                geometry.content_top,
                geometry.catalog_width,
                42.0,
            ),
            terminal_color(248, 250, 252),
            terminal_color(248, 250, 252),
            0.0,
            0.0,
        );
        let title = make_label(
            geometry.rect_from_top_left(
                geometry.catalog_x + 14.0,
                geometry.content_top + 13.0,
                140.0,
                18.0,
            ),
            "Repos",
            13.5,
            terminal_color(15, 23, 42),
        );
        let count = make_label(
            geometry.rect_from_top_left(
                geometry.catalog_x + geometry.catalog_width - 72.0,
                geometry.content_top + 15.0,
                58.0,
                16.0,
            ),
            &format!("{} total", snapshot.repo_catalog.len()),
            10.5,
            terminal_color(100, 116, 139),
        );
        CatalogChrome {
            panel,
            header,
            title,
            count,
        }
    }

    unsafe fn make_terminal_chrome(
        snapshot: &RepoViewSnapshot,
        geometry: NativeViewGeometry,
    ) -> TerminalChrome {
        let (x, y, width, height) = terminal_panel_top_left(snapshot.layout, geometry);
        let panel = make_box(
            geometry.rect_from_top_left(x, y, width, height),
            terminal_color(15, 23, 42),
            terminal_color(51, 65, 85),
            1.0,
            0.0,
        );
        let header = make_box(
            geometry.rect_from_top_left(x, y, width, 42.0),
            terminal_color(30, 41, 59),
            terminal_color(30, 41, 59),
            0.0,
            0.0,
        );
        let title = make_label(
            geometry.rect_from_top_left(x + 16.0, y + 13.0, 160.0, 20.0),
            &snapshot.terminal.title,
            14.0,
            terminal_color(248, 250, 252),
        );
        let badge = make_label(
            geometry.rect_from_top_left(x + width - 126.0, y + 15.0, 110.0, 18.0),
            "agent + repo",
            11.0,
            terminal_color(125, 211, 252),
        );
        TerminalChrome {
            panel,
            header,
            title,
            badge,
        }
    }

    unsafe fn make_detail_chrome(
        snapshot: &RepoViewSnapshot,
        geometry: NativeViewGeometry,
    ) -> DetailChrome {
        let (x, y, width, height) = detail_panel_top_left(snapshot.layout, geometry);
        let panel = make_box(
            geometry.rect_from_top_left(x, y, width, height),
            terminal_color(255, 255, 255),
            terminal_color(203, 213, 225),
            1.0,
            0.0,
        );
        let header_label = make_label(
            geometry.rect_from_top_left(x + 24.0, y + 18.0, 180.0, 18.0),
            "Caps / EC",
            13.0,
            terminal_color(100, 116, 139),
        );
        DetailChrome {
            panel,
            header_label,
        }
    }

    unsafe fn make_project_selector(
        snapshot: &RepoViewSnapshot,
        geometry: NativeViewGeometry,
    ) -> Id {
        let (x, y, width, _) = detail_panel_top_left(snapshot.layout, geometry);
        let popup: Id = msg_send![class!(NSPopUpButton), alloc];
        let popup: Id = msg_send![
            popup,
            initWithFrame: geometry.rect_from_top_left(x + 220.0, y + 12.0, width - 244.0, 30.0)
            pullsDown: NO
        ];
        if snapshot.catalog.is_empty() {
            let _: () = msg_send![popup, addItemWithTitle: ns_string("No project/lib")];
        } else {
            for item in &snapshot.catalog {
                let _: () = msg_send![
                    popup,
                    addItemWithTitle: ns_string(&format!("{} [{}]", item.name, item.kind))
                ];
            }
            if let Some(selected) = snapshot.selected.as_deref() {
                if let Some(item) = snapshot.catalog.iter().find(|item| item.name == selected) {
                    let _: () = msg_send![
                        popup,
                        selectItemWithTitle: ns_string(&format!("{} [{}]", item.name, item.kind))
                    ];
                }
            }
        }
        set_accessibility_identifier(popup, "repo-project-selector");
        popup
    }

    unsafe fn make_box(
        frame: NSRect,
        fill_color: Id,
        border_color: Id,
        border_width: f64,
        corner_radius: f64,
    ) -> Id {
        let box_view: Id = msg_send![class!(NSBox), alloc];
        let box_view: Id = msg_send![box_view, initWithFrame: frame];
        let _: () = msg_send![box_view, setBoxType: NS_BOX_CUSTOM];
        let _: () = msg_send![box_view, setTitlePosition: NS_NO_TITLE];
        let _: () = msg_send![box_view, setBorderType: NS_LINE_BORDER];
        let _: () = msg_send![box_view, setFillColor: fill_color];
        let _: () = msg_send![box_view, setBorderColor: border_color];
        let _: () = msg_send![box_view, setBorderWidth: border_width];
        let _: () = msg_send![box_view, setCornerRadius: corner_radius];
        box_view
    }

    unsafe fn make_label(frame: NSRect, text: &str, font_size: f64, text_color: Id) -> Id {
        let label: Id = msg_send![class!(NSTextField), alloc];
        let label: Id = msg_send![label, initWithFrame: frame];
        let _: () = msg_send![label, setStringValue: ns_string(text)];
        let _: () = msg_send![label, setEditable: NO];
        let _: () = msg_send![label, setSelectable: NO];
        let _: () = msg_send![label, setBordered: NO];
        let _: () = msg_send![label, setBezeled: NO];
        let _: () = msg_send![label, setDrawsBackground: NO];
        let font: Id = msg_send![class!(NSFont), systemFontOfSize: font_size];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setTextColor: text_color];
        label
    }

    unsafe fn make_catalog_views(geometry: NativeViewGeometry) -> (Id, Id) {
        let frame = catalog_frame(geometry);
        make_readonly_text_scroll_view(
            frame,
            terminal_color(255, 255, 255),
            terminal_color(30, 41, 59),
            12.0,
            "repo-catalog",
        )
    }

    unsafe fn make_detail_views(
        snapshot: &RepoViewSnapshot,
        geometry: NativeViewGeometry,
    ) -> (Id, Id) {
        let frame = detail_frame(snapshot.layout, geometry);
        make_readonly_text_scroll_view(
            frame,
            terminal_color(255, 255, 255),
            terminal_color(30, 41, 59),
            12.0,
            "repo-readme-detail",
        )
    }

    unsafe fn make_readonly_text_scroll_view(
        frame: NSRect,
        background: Id,
        text_color: Id,
        font_size: f64,
        accessibility_id: &str,
    ) -> (Id, Id) {
        let scroll_view: Id = msg_send![class!(NSScrollView), alloc];
        let scroll_view: Id = msg_send![scroll_view, initWithFrame: frame];
        let _: () = msg_send![scroll_view, setHasVerticalScroller: YES];
        let _: () = msg_send![scroll_view, setAutohidesScrollers: YES];
        let _: () = msg_send![scroll_view, setDrawsBackground: YES];
        let _: () = msg_send![scroll_view, setBackgroundColor: background];
        set_accessibility_identifier(scroll_view, accessibility_id);

        let output_view: Id = msg_send![class!(NSTextView), alloc];
        let output_view: Id = msg_send![output_view, initWithFrame: rect(0.0, 0.0, frame.size.width, frame.size.height)];
        let _: () = msg_send![output_view, setEditable: NO];
        let _: () = msg_send![output_view, setSelectable: YES];
        let _: () = msg_send![output_view, setVerticallyResizable: YES];
        let _: () = msg_send![output_view, setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE];
        let _: () = msg_send![output_view, setDrawsBackground: YES];
        let _: () = msg_send![output_view, setBackgroundColor: background];
        let _: () = msg_send![output_view, setTextColor: text_color];
        let font: Id = msg_send![class!(NSFont), systemFontOfSize: font_size];
        let _: () = msg_send![output_view, setFont: font];
        let _: () = msg_send![output_view, setTextContainerInset: size(8.0, 8.0)];
        let _: () = msg_send![scroll_view, setDocumentView: output_view];
        (scroll_view, output_view)
    }

    unsafe fn make_terminal_views(
        snapshot: &RepoViewSnapshot,
        geometry: NativeViewGeometry,
    ) -> (Id, Id, Id) {
        let (scroll_frame, output_frame, input_frame) = terminal_frames(snapshot.layout, geometry);

        let scroll_view: Id = msg_send![class!(NSScrollView), alloc];
        let scroll_view: Id = msg_send![scroll_view, initWithFrame: scroll_frame];
        let _: () = msg_send![scroll_view, setHasVerticalScroller: YES];
        let _: () = msg_send![scroll_view, setAutohidesScrollers: YES];
        let _: () = msg_send![scroll_view, setDrawsBackground: YES];
        let _: () = msg_send![scroll_view, setBackgroundColor: terminal_color(15, 23, 42)];
        set_accessibility_identifier(scroll_view, "repo-terminal");

        let output_view: Id = msg_send![class!(NSTextView), alloc];
        let output_view: Id = msg_send![output_view, initWithFrame: output_frame];
        let _: () = msg_send![output_view, setEditable: NO];
        let _: () = msg_send![output_view, setSelectable: YES];
        let _: () = msg_send![output_view, setDrawsBackground: YES];
        let _: () = msg_send![output_view, setBackgroundColor: terminal_color(15, 23, 42)];
        let _: () = msg_send![output_view, setTextColor: terminal_color(226, 232, 240)];
        let font: Id = msg_send![class!(NSFont), userFixedPitchFontOfSize: 12.0];
        let _: () = msg_send![output_view, setFont: font];
        let _: () = msg_send![scroll_view, setDocumentView: output_view];

        let input: Id = msg_send![class!(NSTextField), alloc];
        let input: Id = msg_send![input, initWithFrame: input_frame];
        let _: () = msg_send![input, setFont: font];
        let _: () = msg_send![input, setTextColor: terminal_color(134, 239, 172)];
        let _: () = msg_send![input, setBackgroundColor: terminal_color(15, 23, 42)];
        let _: () = msg_send![input, setDrawsBackground: YES];
        let _: () = msg_send![input, setBordered: NO];
        let _: () = msg_send![input, setBezeled: NO];
        let _: () =
            msg_send![input, setPlaceholderString: ns_string("type command and press Return")];
        set_accessibility_identifier(input, "repo-terminal-input");

        (scroll_view, output_view, input)
    }

    unsafe fn make_layout_toggle_button(
        snapshot: &RepoViewSnapshot,
        geometry: NativeViewGeometry,
    ) -> Id {
        let frame = layout_toggle_button_frame(geometry);
        let button: Id = msg_send![class!(NSButton), alloc];
        let button: Id = msg_send![button, initWithFrame: frame];
        let _: () = msg_send![button, setBezelStyle: NS_BEZEL_STYLE_ROUNDED];
        let _: () = msg_send![button, setAutoresizingMask: top_right_autoresize_mask()];
        let font: Id = msg_send![class!(NSFont), systemFontOfSize: 12.0];
        let _: () = msg_send![button, setFont: font];
        set_accessibility_identifier(button, "repo-layout-toggle");
        set_layout_toggle_button_title(button, snapshot);
        button
    }

    unsafe fn make_layout_toggle_controller(state: *mut c_void) -> Id {
        let class = layout_toggle_controller_class();
        let controller: Id = msg_send![class, alloc];
        let controller: Id = msg_send![controller, init];
        (*controller).set_ivar("state", state);
        controller
    }

    unsafe fn observe_window_resize(controller: Id, window: Id) {
        let center: Id = msg_send![class!(NSNotificationCenter), defaultCenter];
        let _: () = msg_send![
            center,
            addObserver: controller
            selector: sel!(windowDidResize:)
            name: ns_string("NSWindowDidResizeNotification")
            object: window
        ];
    }

    fn layout_toggle_controller_class() -> *const Class {
        static REGISTER: Once = Once::new();
        static mut CLASS: *const Class = ptr::null();

        unsafe {
            REGISTER.call_once(|| {
                let superclass = class!(NSObject);
                let mut declaration =
                    ClassDecl::new("AwRepoViewLayoutToggleController", superclass)
                        .expect("AwRepoViewLayoutToggleController class declaration");
                declaration.add_ivar::<*mut c_void>("state");
                declaration.add_method(
                    sel!(toggleLayout:),
                    toggle_layout as extern "C" fn(&Object, Sel, Id),
                );
                declaration.add_method(
                    sel!(runTerminalCommand:),
                    run_terminal_command as extern "C" fn(&Object, Sel, Id),
                );
                declaration.add_method(
                    sel!(selectProject:),
                    select_project as extern "C" fn(&Object, Sel, Id),
                );
                declaration.add_method(
                    sel!(scheduleTerminalFlush:),
                    schedule_terminal_flush as extern "C" fn(&Object, Sel, Id),
                );
                declaration.add_method(
                    sel!(flushScheduledRedraw:),
                    flush_scheduled_redraw as extern "C" fn(&Object, Sel, Id),
                );
                declaration.add_method(
                    sel!(windowDidResize:),
                    window_did_resize as extern "C" fn(&Object, Sel, Id),
                );
                CLASS = declaration.register();
            });
            CLASS
        }
    }

    extern "C" fn toggle_layout(this: &Object, _selector: Sel, _sender: Id) {
        unsafe {
            let state_ptr: *mut c_void = *this.get_ivar("state");
            if state_ptr.is_null() {
                return;
            }
            let state = &mut *(state_ptr.cast::<NativeRepoViewState>());
            state.snapshot.layout = toggled_view_layout(state.snapshot.layout);
            let controller = this as *const Object as Id;
            schedule_redraw(controller, state, RedrawReason::LayoutToggle);
        }
    }

    extern "C" fn select_project(this: &Object, _selector: Sel, sender: Id) {
        unsafe {
            let state_ptr: *mut c_void = *this.get_ivar("state");
            if state_ptr.is_null() {
                return;
            }
            let state = &mut *(state_ptr.cast::<NativeRepoViewState>());
            let index: isize = msg_send![sender, indexOfSelectedItem];
            if select_project_by_selector_index(state, index) {
                let controller = this as *const Object as Id;
                schedule_redraw(controller, state, RedrawReason::ProjectSelection);
            }
        }
    }

    extern "C" fn window_did_resize(this: &Object, _selector: Sel, _sender: Id) {
        unsafe {
            let state_ptr: *mut c_void = *this.get_ivar("state");
            if state_ptr.is_null() {
                return;
            }
            let state = &mut *(state_ptr.cast::<NativeRepoViewState>());
            let controller = this as *const Object as Id;
            schedule_redraw(controller, state, RedrawReason::WindowResize);
        }
    }

    extern "C" fn run_terminal_command(this: &Object, _selector: Sel, sender: Id) {
        unsafe {
            let state_ptr: *mut c_void = *this.get_ivar("state");
            if state_ptr.is_null() {
                return;
            }
            let state = &mut *(state_ptr.cast::<NativeRepoViewState>());
            let command = string_value(sender);
            let _: () = msg_send![sender, setStringValue: ns_string("")];
            let controller = this as *const Object as Id;
            if apply_terminal_command(state, command.trim()) {
                schedule_redraw(controller, state, RedrawReason::TerminalInput);
            }
        }
    }

    extern "C" fn schedule_terminal_flush(this: &Object, _selector: Sel, _sender: Id) {
        unsafe {
            let state_ptr: *mut c_void = *this.get_ivar("state");
            if state_ptr.is_null() {
                return;
            }
            let state = &mut *(state_ptr.cast::<NativeRepoViewState>());
            let controller = this as *const Object as Id;
            schedule_redraw(controller, state, RedrawReason::TerminalOutput);
        }
    }

    extern "C" fn flush_scheduled_redraw(this: &Object, _selector: Sel, _sender: Id) {
        unsafe {
            let state_ptr: *mut c_void = *this.get_ivar("state");
            if state_ptr.is_null() {
                return;
            }
            let state = &mut *(state_ptr.cast::<NativeRepoViewState>());
            let frame = state.render_scheduler.take_frame();
            flush_render_frame(state, frame);
        }
    }

    unsafe fn schedule_redraw(
        controller: Id,
        state: &mut NativeRepoViewState,
        reason: RedrawReason,
    ) {
        if !state.render_scheduler.request_redraw(reason) {
            return;
        }
        let _: Id = msg_send![
            class!(NSTimer),
            scheduledTimerWithTimeInterval: state.render_scheduler.coalescing_interval()
            target: controller
            selector: sel!(flushScheduledRedraw:)
            userInfo: ptr::null_mut::<Object>()
            repeats: NO
        ];
    }

    unsafe fn flush_render_frame(state: &mut NativeRepoViewState, frame: RenderFrame) {
        let terminal_changed = drain_terminal_events(state);
        if frame.redraw_controls {
            set_layout_toggle_button_title(state.button, &state.snapshot);
            layout_chrome_views(state);
            layout_project_views(state);
            layout_terminal_views(state);
            set_detail_output(state);
            let _: () = msg_send![state.button, setNeedsDisplay: YES];
        }
        if frame.redraw_terminal || terminal_changed {
            set_terminal_output(state);
        }
    }

    unsafe fn set_layout_toggle_button_title(button: Id, snapshot: &RepoViewSnapshot) {
        let title = ns_string(layout_toggle_button_label(snapshot.layout));
        let _: () = msg_send![button, setTitle: title];
    }

    unsafe fn main_screen_maximum_frames_per_second() -> f64 {
        let screen: Id = msg_send![class!(NSScreen), mainScreen];
        if screen.is_null() {
            return 60.0;
        }
        let responds: bool = msg_send![screen, respondsToSelector: sel!(maximumFramesPerSecond)];
        if !responds {
            return 60.0;
        }
        let frames_per_second: isize = msg_send![screen, maximumFramesPerSecond];
        if frames_per_second > 0 {
            (frames_per_second as f64).clamp(30.0, 240.0)
        } else {
            60.0
        }
    }

    unsafe fn current_geometry(state: &NativeRepoViewState) -> NativeViewGeometry {
        let bounds: NSRect = msg_send![state.root_view, bounds];
        NativeViewGeometry::from_size(bounds.size)
    }

    unsafe fn layout_terminal_views(state: &NativeRepoViewState) {
        let geometry = current_geometry(state);
        let (scroll_frame, output_frame, input_frame) =
            terminal_frames(state.snapshot.layout, geometry);
        let _: () = msg_send![state.terminal_scroll_view, setFrame: scroll_frame];
        let _: () = msg_send![state.terminal_output_view, setFrame: output_frame];
        let _: () = msg_send![state.terminal_input, setFrame: input_frame];
        let _: () = msg_send![state.terminal_scroll_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.terminal_input, setNeedsDisplay: YES];
    }

    unsafe fn layout_chrome_views(state: &NativeRepoViewState) {
        let geometry = current_geometry(state);
        let _: () = msg_send![state.root_background_view, setFrame: geometry.root_frame()];
        let _: () = msg_send![
            state.header_view,
            setFrame: geometry.rect_from_top_left(0.0, 0.0, geometry.width, 58.0)
        ];
        let _: () = msg_send![
            state.header_rule_view,
            setFrame: geometry.rect_from_top_left(0.0, 58.0, geometry.width, 1.0)
        ];
        let _: () = msg_send![
            state.header_title_label,
            setFrame: geometry.rect_from_top_left(28.0, 17.0, 170.0, 28.0)
        ];
        let _: () = msg_send![
            state.header_summary_label,
            setFrame: geometry.rect_from_top_left(
                210.0,
                20.0,
                (geometry.width - 460.0).max(120.0),
                22.0
            )
        ];
        let _: () = msg_send![state.catalog_panel_view, setFrame: catalog_panel_frame(geometry)];
        let _: () = msg_send![
            state.catalog_header_view,
            setFrame: geometry.rect_from_top_left(
                geometry.catalog_x,
                geometry.content_top,
                geometry.catalog_width,
                42.0
            )
        ];
        let _: () = msg_send![
            state.catalog_title_label,
            setFrame: geometry.rect_from_top_left(
                geometry.catalog_x + 14.0,
                geometry.content_top + 13.0,
                140.0,
                18.0
            )
        ];
        let _: () = msg_send![
            state.catalog_count_label,
            setFrame: geometry.rect_from_top_left(
                geometry.catalog_x + geometry.catalog_width - 72.0,
                geometry.content_top + 15.0,
                58.0,
                16.0
            )
        ];
        let (terminal_x, terminal_y, terminal_w, terminal_h) =
            terminal_panel_top_left(state.snapshot.layout, geometry);
        let _: () = msg_send![
            state.terminal_panel_view,
            setFrame: geometry.rect_from_top_left(terminal_x, terminal_y, terminal_w, terminal_h)
        ];
        let _: () = msg_send![
            state.terminal_header_view,
            setFrame: geometry.rect_from_top_left(terminal_x, terminal_y, terminal_w, 42.0)
        ];
        let _: () = msg_send![
            state.terminal_title_label,
            setFrame: geometry.rect_from_top_left(
                terminal_x + 16.0,
                terminal_y + 13.0,
                160.0,
                20.0
            )
        ];
        let _: () = msg_send![
            state.terminal_badge_label,
            setFrame: geometry.rect_from_top_left(
                terminal_x + terminal_w - 126.0,
                terminal_y + 15.0,
                110.0,
                18.0
            )
        ];

        let (detail_x, detail_y, detail_w, detail_h) =
            detail_panel_top_left(state.snapshot.layout, geometry);
        let _: () = msg_send![
            state.detail_panel_view,
            setFrame: geometry.rect_from_top_left(detail_x, detail_y, detail_w, detail_h)
        ];
        let _: () = msg_send![
            state.detail_header_label,
            setFrame: geometry.rect_from_top_left(detail_x + 24.0, detail_y + 18.0, 180.0, 18.0)
        ];
        let _: () = msg_send![
            state.project_selector,
            setFrame: geometry.rect_from_top_left(
                detail_x + 220.0,
                detail_y + 12.0,
                detail_w - 244.0,
                30.0
            )
        ];
        let _: () = msg_send![state.button, setFrame: layout_toggle_button_frame(geometry)];
        let _: () = msg_send![state.root_background_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.header_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.header_rule_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.catalog_panel_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.catalog_header_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.terminal_panel_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.terminal_header_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.detail_panel_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.project_selector, setNeedsDisplay: YES];
    }

    unsafe fn layout_project_views(state: &NativeRepoViewState) {
        let geometry = current_geometry(state);
        let catalog_frame = catalog_frame(geometry);
        let detail_frame = detail_frame(state.snapshot.layout, geometry);
        let _: () = msg_send![state.catalog_scroll_view, setFrame: catalog_frame];
        let _: () = msg_send![state.detail_scroll_view, setFrame: detail_frame];
        let _: () = msg_send![state.catalog_scroll_view, setNeedsDisplay: YES];
        let _: () = msg_send![state.detail_scroll_view, setNeedsDisplay: YES];
    }

    unsafe fn set_catalog_output(state: &mut NativeRepoViewState) {
        set_text_view_string(
            state.catalog_text_view,
            &catalog_text(&state.snapshot),
            false,
        );
    }

    unsafe fn set_detail_output(state: &mut NativeRepoViewState) {
        set_text_view_string(state.detail_text_view, &detail_text(&state.snapshot), false);
    }

    unsafe fn set_terminal_output(state: &mut NativeRepoViewState) {
        set_text_view_string(state.terminal_output_view, &state.terminal_log, true);
    }

    fn select_project_by_selector_index(state: &mut NativeRepoViewState, index: isize) -> bool {
        if index < 0 {
            return false;
        }
        let Some(item) = state.snapshot.catalog.get(index as usize) else {
            return false;
        };
        if state.snapshot.selected.as_deref() == Some(item.name.as_str()) {
            return false;
        }
        state.snapshot.selected = Some(item.name.clone());
        true
    }

    unsafe fn set_text_view_string(text_view: Id, value: &str, scroll_to_end: bool) {
        let value = ns_string(value);
        let _: () = msg_send![text_view, setString: value];
        let length: usize = msg_send![value, length];
        let range = NSRange {
            location: if scroll_to_end { length } else { 0 },
            length: 0,
        };
        let _: () = msg_send![text_view, scrollRangeToVisible: range];
    }

    fn apply_terminal_command(state: &mut NativeRepoViewState, command: &str) -> bool {
        if command.is_empty() {
            state.terminal_log.push('\n');
            return true;
        }

        if command == "clear" {
            state.terminal_log = initial_terminal_log(&state.snapshot);
            return true;
        }

        let Some(session) = state.terminal_session.as_ref() else {
            state
                .terminal_log
                .push_str("terminal PTY session is not available\n");
            return true;
        };
        if let Err(err) = session.write_command(command) {
            state
                .terminal_log
                .push_str(&format!("terminal PTY write failed: {err}\n"));
        }
        true
    }

    fn initial_terminal_log(snapshot: &RepoViewSnapshot) -> String {
        let mut log = snapshot.terminal.lines.join("\n");
        log.push_str("\n\n# PTY shell: type a command below and press Return\n");
        log
    }

    fn start_terminal_pty_session(
        cwd: PathBuf,
        tx: Sender<TerminalEvent>,
        controller: Id,
    ) -> std::io::Result<TerminalPtySession> {
        let cwd_c = CString::new(cwd.as_os_str().as_bytes()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "terminal cwd contains a NUL byte",
            )
        })?;
        let shell = CString::new("/bin/zsh").expect("static shell path has no NUL");
        let argv0 = CString::new("zsh").expect("static argv has no NUL");
        let fast_start = CString::new("-f").expect("static argv has no NUL");
        let term_key = CString::new("TERM").expect("static env key has no NUL");
        let term_value = CString::new("xterm-256color").expect("static env value has no NUL");
        let prompt_key = CString::new("PROMPT").expect("static env key has no NUL");
        let prompt_value = CString::new("$ ").expect("static env value has no NUL");
        let ps1_key = CString::new("PS1").expect("static env key has no NUL");
        let mut master_fd: libc::c_int = -1;
        let mut window_size = libc::winsize {
            ws_row: 40,
            ws_col: 120,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let child_pid = unsafe {
            libc::forkpty(
                &mut master_fd,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut window_size,
            )
        };
        if child_pid < 0 {
            return Err(std::io::Error::last_os_error());
        }
        if child_pid == 0 {
            unsafe {
                let _ = libc::chdir(cwd_c.as_ptr());
                let _ = libc::setenv(term_key.as_ptr(), term_value.as_ptr(), 1);
                let _ = libc::setenv(prompt_key.as_ptr(), prompt_value.as_ptr(), 1);
                let _ = libc::setenv(ps1_key.as_ptr(), prompt_value.as_ptr(), 1);
                libc::execl(
                    shell.as_ptr(),
                    argv0.as_ptr(),
                    fast_start.as_ptr(),
                    ptr::null::<libc::c_char>(),
                );
                libc::_exit(127);
            }
        }

        let writer_fd = unsafe { libc::dup(master_fd) };
        if writer_fd < 0 {
            unsafe {
                let _ = libc::close(master_fd);
                let _ = libc::kill(child_pid, libc::SIGHUP);
            }
            return Err(std::io::Error::last_os_error());
        }
        let reader = unsafe { File::from_raw_fd(master_fd) };
        let writer = Arc::new(Mutex::new(unsafe { File::from_raw_fd(writer_fd) }));
        spawn_pty_reader(reader, child_pid, tx, controller);
        Ok(TerminalPtySession { writer, child_pid })
    }

    fn spawn_pty_reader(
        mut reader: File,
        child_pid: libc::pid_t,
        tx: Sender<TerminalEvent>,
        controller: Id,
    ) {
        let controller_addr = controller as usize;
        thread::spawn(move || {
            let wake = move || unsafe { wake_controller_for_terminal_flush(controller_addr) };
            let mut buffer = [0_u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => send_terminal_event(
                        &tx,
                        TerminalEvent::Output {
                            stream: TerminalStream::Pty,
                            text: String::from_utf8_lossy(&buffer[..n]).into_owned(),
                        },
                        &wake,
                    ),
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(err) => {
                        send_terminal_event(
                            &tx,
                            TerminalEvent::Output {
                                stream: TerminalStream::System,
                                text: format!("terminal PTY read failed: {err}\n"),
                            },
                            &wake,
                        );
                        break;
                    }
                }
            }
            let mut status: libc::c_int = 0;
            let wait_result = unsafe { libc::waitpid(child_pid, &mut status, 0) };
            if wait_result == child_pid {
                let code = if libc::WIFEXITED(status) {
                    Some(libc::WEXITSTATUS(status))
                } else if libc::WIFSIGNALED(status) {
                    Some(128 + libc::WTERMSIG(status))
                } else {
                    None
                };
                send_terminal_event(&tx, TerminalEvent::Exit { code }, &wake);
            }
        });
    }

    fn send_terminal_event<W>(tx: &Sender<TerminalEvent>, event: TerminalEvent, wake: &W)
    where
        W: Fn() + ?Sized,
    {
        if tx.send(event).is_ok() {
            wake();
        }
    }

    fn drain_terminal_events(state: &mut NativeRepoViewState) -> bool {
        let mut changed = false;
        while let Ok(event) = state.terminal_rx.try_recv() {
            match event {
                TerminalEvent::Output { stream, text } => {
                    let _ = stream;
                    state
                        .terminal_log
                        .push_str(&strip_terminal_control_sequences(&text));
                    changed = true;
                }
                TerminalEvent::Exit { code } => {
                    if let Some(code) = code {
                        if code != 0 {
                            state.terminal_log.push_str(&format!("[exit {code}]\n"));
                            changed = true;
                        }
                    }
                }
            }
        }
        changed
    }

    fn strip_terminal_control_sequences(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                match chars.next() {
                    Some('[') => consume_csi_sequence(&mut chars),
                    Some(']') => consume_osc_sequence(&mut chars),
                    Some('(') | Some(')') | Some('*') | Some('+') => {
                        let _ = chars.next();
                    }
                    Some(_) | None => {}
                }
                continue;
            }
            match ch {
                '\n' | '\t' => out.push(ch),
                '\r' => {}
                ch if ch.is_control() => {}
                _ => out.push(ch),
            }
        }
        out
    }

    fn consume_csi_sequence<I>(chars: &mut std::iter::Peekable<I>)
    where
        I: Iterator<Item = char>,
    {
        for ch in chars.by_ref() {
            if ('\u{40}'..='\u{7e}').contains(&ch) {
                break;
            }
        }
    }

    fn consume_osc_sequence<I>(chars: &mut std::iter::Peekable<I>)
    where
        I: Iterator<Item = char>,
    {
        while let Some(ch) = chars.next() {
            if ch == '\u{7}' {
                break;
            }
            if ch == '\u{1b}' && chars.peek() == Some(&'\\') {
                let _ = chars.next();
                break;
            }
        }
    }

    unsafe fn wake_controller_for_terminal_flush(controller_addr: usize) {
        let controller = controller_addr as Id;
        if controller.is_null() {
            return;
        }
        let _: () = msg_send![
            controller,
            performSelectorOnMainThread: sel!(scheduleTerminalFlush:)
            withObject: ptr::null_mut::<Object>()
            waitUntilDone: NO
        ];
    }

    unsafe fn string_value(object: Id) -> String {
        let value: Id = msg_send![object, stringValue];
        ns_string_to_string(value)
    }

    unsafe fn ns_string_to_string(value: Id) -> String {
        if value.is_null() {
            return String::new();
        }
        let bytes: *const c_char = msg_send![value, UTF8String];
        if bytes.is_null() {
            return String::new();
        }
        CStr::from_ptr(bytes).to_string_lossy().into_owned()
    }

    unsafe fn terminal_color(red: u8, green: u8, blue: u8) -> Id {
        msg_send![
            class!(NSColor),
            colorWithCalibratedRed: red as f64 / 255.0
            green: green as f64 / 255.0
            blue: blue as f64 / 255.0
            alpha: 1.0
        ]
    }

    unsafe fn set_accessibility_identifier(view: Id, identifier: &str) {
        let _: () = msg_send![view, setAccessibilityIdentifier: ns_string(identifier)];
    }

    fn catalog_panel_frame(geometry: NativeViewGeometry) -> NSRect {
        geometry.rect_from_top_left(
            geometry.catalog_x,
            geometry.content_top,
            geometry.catalog_width,
            geometry.content_height,
        )
    }

    fn catalog_frame(geometry: NativeViewGeometry) -> NSRect {
        content_frame_from_top_left(
            geometry,
            geometry.catalog_x,
            geometry.content_top,
            geometry.catalog_width,
            geometry.content_height,
            42.0,
            8.0,
            12.0,
            12.0,
        )
    }

    fn detail_frame(layout: crate::cli::view::ViewLayout, geometry: NativeViewGeometry) -> NSRect {
        let (x, y, width, height) = detail_panel_top_left(layout, geometry);
        content_frame_from_top_left(geometry, x, y, width, height, 88.0, 10.0, 24.0, 24.0)
    }

    fn terminal_panel_top_left(
        layout: crate::cli::view::ViewLayout,
        geometry: NativeViewGeometry,
    ) -> (f64, f64, f64, f64) {
        match layout {
            crate::cli::view::ViewLayout::LeftRight => {
                let terminal_width = (geometry.right_width * 0.39).clamp(360.0, 640.0);
                (
                    geometry.right_x,
                    geometry.content_top,
                    terminal_width,
                    geometry.content_height,
                )
            }
            crate::cli::view::ViewLayout::TopBottom => {
                let terminal_height = (geometry.content_height * 0.30).clamp(180.0, 300.0);
                (
                    geometry.right_x,
                    geometry.content_top,
                    geometry.right_width,
                    terminal_height,
                )
            }
        }
    }

    fn detail_panel_top_left(
        layout: crate::cli::view::ViewLayout,
        geometry: NativeViewGeometry,
    ) -> (f64, f64, f64, f64) {
        match layout {
            crate::cli::view::ViewLayout::LeftRight => {
                let (terminal_x, terminal_y, terminal_width, terminal_height) =
                    terminal_panel_top_left(layout, geometry);
                (
                    terminal_x + terminal_width + geometry.gap,
                    terminal_y,
                    (geometry.right_width - terminal_width - geometry.gap).max(360.0),
                    terminal_height,
                )
            }
            crate::cli::view::ViewLayout::TopBottom => {
                let (_, terminal_y, _, terminal_height) = terminal_panel_top_left(layout, geometry);
                let detail_y = terminal_y + terminal_height + geometry.gap;
                (
                    geometry.right_x,
                    detail_y,
                    geometry.right_width,
                    (geometry.content_top + geometry.content_height - detail_y).max(260.0),
                )
            }
        }
    }

    fn content_frame_from_top_left(
        geometry: NativeViewGeometry,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        header_height: f64,
        bottom_padding: f64,
        horizontal_padding: f64,
        right_padding: f64,
    ) -> NSRect {
        let content_top_y = y + header_height;
        let content_height = (height - header_height - bottom_padding).max(1.0);
        let appkit_y = geometry.height - content_top_y - content_height;
        rect(
            x + horizontal_padding,
            appkit_y,
            (width - horizontal_padding - right_padding).max(1.0),
            content_height,
        )
    }

    fn terminal_frames(
        layout: crate::cli::view::ViewLayout,
        geometry: NativeViewGeometry,
    ) -> (NSRect, NSRect, NSRect) {
        let (x, y, width, height) = terminal_panel_top_left(layout, geometry);
        let header_height = 42.0;
        let padding = 16.0;
        let input_height = 26.0;
        let gap = 10.0;
        let content_top_y = y + header_height;
        let content_height = height - header_height;
        let content_bottom_y = geometry.height - content_top_y - content_height;
        let scroll_x = x + padding;
        let scroll_y = content_bottom_y + input_height + gap + 8.0;
        let scroll_w = (width - padding * 2.0).max(1.0);
        let scroll_h = (content_height - input_height - gap - 18.0).max(1.0);
        let input_x = x + padding;
        let input_y = content_bottom_y + 8.0;
        let input_w = scroll_w;
        let scroll_frame = rect(scroll_x, scroll_y, scroll_w, scroll_h);
        let output_frame = rect(0.0, 0.0, scroll_w, scroll_h);
        let input_frame = rect(input_x, input_y, input_w, input_height);
        (scroll_frame, output_frame, input_frame)
    }

    fn layout_toggle_button_frame(geometry: NativeViewGeometry) -> NSRect {
        geometry.rect_from_top_left(geometry.width - 218.0, 14.0, 190.0, 30.0)
    }

    unsafe fn ns_string(value: &str) -> Id {
        let string: Id = msg_send![class!(NSString), alloc];
        msg_send![
            string,
            initWithBytes: value.as_ptr()
            length: value.len()
            encoding: NS_UTF8_STRING_ENCODING
        ]
    }

    fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
        NSRect {
            origin: NSPoint { x, y },
            size: NSSize { width, height },
        }
    }

    fn size(width: f64, height: f64) -> NSSize {
        NSSize { width, height }
    }

    fn autoresize_mask() -> usize {
        NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
    }

    fn top_right_autoresize_mask() -> usize {
        NS_VIEW_MIN_X_MARGIN | NS_VIEW_MIN_Y_MARGIN
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::cli::view::{RepoViewRepo, TerminalSnapshot, ViewLayout};
        use std::time::{Duration, Instant};

        #[test]
        fn terminal_pty_executes_in_repo_root_and_persists_cd() {
            let tmp = tempfile::TempDir::new().unwrap();
            let nested = tmp.path().join("nested");
            std::fs::create_dir(&nested).unwrap();
            let snapshot = RepoViewSnapshot {
                schema_version: 1,
                layout: ViewLayout::LeftRight,
                repo: RepoViewRepo {
                    name: "repo".to_string(),
                    root: tmp.path().display().to_string(),
                    item_count: 0,
                    project_count: 0,
                    library_count: 0,
                },
                repo_catalog: vec![crate::cli::view::RepoCatalogItem {
                    name: "repo".to_string(),
                    path: tmp.path().display().to_string(),
                    current: true,
                    item_count: 0,
                    project_count: 0,
                    library_count: 0,
                }],
                selected_repo: Some(tmp.path().display().to_string()),
                terminal: TerminalSnapshot {
                    title: "Terminal".to_string(),
                    lines: vec!["$ aw view".to_string()],
                },
                catalog: Vec::new(),
                selected: None,
                items: Vec::new(),
                surface: cclab_surface::Element::intrinsic(
                    "main",
                    cclab_surface::Props::default(),
                    Vec::new(),
                )
                .surface_snapshot(),
                warnings: Vec::new(),
            };
            let (terminal_tx, terminal_rx) = mpsc::channel();
            let mut state = NativeRepoViewState {
                snapshot,
                root_view: ptr::null_mut(),
                root_background_view: ptr::null_mut(),
                header_view: ptr::null_mut(),
                header_rule_view: ptr::null_mut(),
                header_title_label: ptr::null_mut(),
                header_summary_label: ptr::null_mut(),
                catalog_panel_view: ptr::null_mut(),
                catalog_header_view: ptr::null_mut(),
                catalog_title_label: ptr::null_mut(),
                catalog_count_label: ptr::null_mut(),
                button: ptr::null_mut(),
                controller: ptr::null_mut(),
                terminal_panel_view: ptr::null_mut(),
                terminal_header_view: ptr::null_mut(),
                terminal_title_label: ptr::null_mut(),
                terminal_badge_label: ptr::null_mut(),
                detail_panel_view: ptr::null_mut(),
                detail_header_label: ptr::null_mut(),
                project_selector: ptr::null_mut(),
                catalog_scroll_view: ptr::null_mut(),
                catalog_text_view: ptr::null_mut(),
                detail_scroll_view: ptr::null_mut(),
                detail_text_view: ptr::null_mut(),
                terminal_scroll_view: ptr::null_mut(),
                terminal_output_view: ptr::null_mut(),
                terminal_input: ptr::null_mut(),
                terminal_tx,
                terminal_rx,
                terminal_session: None,
                terminal_cwd: PathBuf::from(tmp.path()),
                terminal_log: String::new(),
                render_scheduler: RenderScheduler::new(FrameClock {
                    frames_per_second: 60.0,
                }),
            };

            state.terminal_session = Some(
                start_terminal_pty_session(
                    PathBuf::from(tmp.path()),
                    state.terminal_tx.clone(),
                    ptr::null_mut(),
                )
                .unwrap(),
            );
            assert!(apply_terminal_command(&mut state, "pwd"));
            drain_until_terminal_log_contains(
                &mut state,
                &tmp.path().display().to_string(),
                Duration::from_secs(3),
            );
            assert!(apply_terminal_command(&mut state, "cd nested"));
            assert!(apply_terminal_command(
                &mut state,
                "pwd && printf aw-terminal-ok"
            ));
            drain_until_terminal_log_contains(&mut state, "aw-terminal-ok", Duration::from_secs(3));
            assert!(state
                .terminal_log
                .contains(&nested.canonicalize().unwrap().display().to_string()));
        }

        fn drain_until_terminal_log_contains(
            state: &mut NativeRepoViewState,
            needle: &str,
            timeout: Duration,
        ) {
            let deadline = Instant::now() + timeout;
            while Instant::now() < deadline {
                drain_terminal_events(state);
                if state.terminal_log.contains(needle) {
                    return;
                }
                thread::sleep(Duration::from_millis(20));
            }
            drain_terminal_events(state);
            panic!(
                "terminal log did not contain `{needle}` before timeout:\n{}",
                state.terminal_log
            );
        }

        #[test]
        fn terminal_pty_reports_command_output_through_queue() {
            let tmp = tempfile::TempDir::new().unwrap();
            let (tx, rx) = mpsc::channel();
            let session =
                start_terminal_pty_session(PathBuf::from(tmp.path()), tx.clone(), ptr::null_mut())
                    .unwrap();
            session.write_command("printf aw-pty-queue-ok").unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut output = String::new();
            while Instant::now() < deadline {
                while let Ok(event) = rx.try_recv() {
                    if let TerminalEvent::Output { text, .. } = event {
                        output.push_str(&text);
                    }
                }
                if output.contains("aw-pty-queue-ok") {
                    return;
                }
                thread::sleep(Duration::from_millis(20));
            }
            panic!("PTY queue did not receive command output: {output}");
        }

        #[test]
        fn terminal_output_sanitizer_removes_ansi_control_sequences() {
            let raw = "\u{1b}[1mhello\u{1b}[0m\r\n\u{1b}]0;title\u{7}world\u{1b}[?2004h";
            assert_eq!(strip_terminal_control_sequences(raw), "hello\nworld");
        }

        #[test]
        fn render_scheduler_coalesces_multiple_redraw_requests() {
            let mut scheduler = RenderScheduler::new(FrameClock {
                frames_per_second: 120.0,
            });

            assert!(scheduler.request_redraw(RedrawReason::TerminalInput));
            assert!(!scheduler.request_redraw(RedrawReason::TerminalOutput));
            assert!(!scheduler.request_redraw(RedrawReason::TerminalOutput));

            let frame = scheduler.take_frame();
            assert!(frame.redraw_terminal);
            assert!(!frame.redraw_controls);
            assert_eq!(scheduler.frame_count, 1);
            assert!(scheduler.request_redraw(RedrawReason::LayoutToggle));
        }

        #[test]
        fn native_view_geometry_expands_for_fullscreen_bounds() {
            let initial = NativeViewGeometry::initial();
            let fullscreen = NativeViewGeometry::from_size(size(1920.0, 1080.0));

            assert!(fullscreen.width > initial.width);
            assert!(fullscreen.height > initial.height);
            assert!(fullscreen.content_height > initial.content_height);
            assert!(fullscreen.right_width > initial.right_width);

            let (_, _, initial_terminal_width, initial_terminal_height) =
                terminal_panel_top_left(ViewLayout::LeftRight, initial);
            let (_, _, fullscreen_terminal_width, fullscreen_terminal_height) =
                terminal_panel_top_left(ViewLayout::LeftRight, fullscreen);
            assert!(fullscreen_terminal_width >= initial_terminal_width);
            assert!(fullscreen_terminal_height > initial_terminal_height);

            let (_, _, initial_detail_width, initial_detail_height) =
                detail_panel_top_left(ViewLayout::LeftRight, initial);
            let (_, _, fullscreen_detail_width, fullscreen_detail_height) =
                detail_panel_top_left(ViewLayout::LeftRight, fullscreen);
            assert!(fullscreen_detail_width > initial_detail_width);
            assert!(fullscreen_detail_height > initial_detail_height);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_view_text_contains_repo_catalog_and_detail() {
        let snapshot = RepoViewSnapshot {
            schema_version: 1,
            layout: crate::cli::view::ViewLayout::LeftRight,
            repo: crate::cli::view::RepoViewRepo {
                name: "repo".to_string(),
                root: "/tmp/repo".to_string(),
                item_count: 1,
                project_count: 1,
                library_count: 0,
            },
            repo_catalog: vec![crate::cli::view::RepoCatalogItem {
                name: "repo".to_string(),
                path: "/tmp/repo".to_string(),
                current: true,
                item_count: 1,
                project_count: 1,
                library_count: 0,
            }],
            selected_repo: Some("/tmp/repo".to_string()),
            terminal: crate::cli::view::TerminalSnapshot {
                title: "Terminal".to_string(),
                lines: vec![
                    "$ aw view".to_string(),
                    "selected agentic-workflow".to_string(),
                ],
            },
            catalog: vec![crate::cli::view::ProjectCatalogItem {
                name: "agentic-workflow".to_string(),
                aliases: vec!["aw".to_string()],
                kind: "project".to_string(),
                path: "projects/agentic-workflow".to_string(),
                cap_path: Some("projects/agentic-workflow/CAPABILITIES.md".to_string()),
            }],
            selected: Some("agentic-workflow".to_string()),
            items: vec![RepoViewItemSnapshot {
                project: crate::cli::view::ProjectViewProject {
                    name: "agentic-workflow".to_string(),
                    aliases: vec!["aw".to_string()],
                    kind: "project".to_string(),
                    path: "projects/agentic-workflow".to_string(),
                    td_path: "projects/agentic-workflow/tech-design".to_string(),
                    cap_path: "projects/agentic-workflow/CAPABILITIES.md".to_string(),
                    label: "project:agentic-workflow".to_string(),
                },
                readme: crate::cli::view::ReadmeSnapshot {
                    path: "projects/agentic-workflow/README.md".to_string(),
                    title: "Agentic Workflow".to_string(),
                    brief: "Workflow protocol.".to_string(),
                    format_version: 2,
                    finding_count: 0,
                },
                capabilities: crate::cli::view::CapabilitySnapshot {
                    count: 1,
                    items: vec![crate::cli::view::CapabilitySnapshotItem {
                        id: "repo-view-desktop-app".to_string(),
                        title: "Repo View Desktop App".to_string(),
                        status: "verified".to_string(),
                        capability_type: Some("DeveloperTool".to_string()),
                        surface_count: 1,
                        ec_dimension_count: 1,
                        claim_count: 1,
                        td_ref_count: 1,
                        ec_case_count: 1,
                    }],
                },
                ec: crate::cli::view::EcSnapshot {
                    inventory_path: "projects/agentic-workflow/aw.toml".to_string(),
                    present: true,
                    generated: true,
                    case_count: 1,
                    production_case_count: 1,
                    by_category: Default::default(),
                    cases: Vec::new(),
                },
                td: crate::cli::view::TdSnapshot {
                    root: "projects/agentic-workflow/tech-design".to_string(),
                    markdown_file_count: 3,
                    capability_ref_count: 2,
                },
                warnings: Vec::new(),
            }],
            surface: cclab_surface::Element::intrinsic(
                "main",
                cclab_surface::Props {
                    id: Some("aw-view".to_string()),
                    ..Default::default()
                },
                Vec::new(),
            )
            .surface_snapshot(),
            warnings: Vec::new(),
        };

        let catalog = catalog_text(&snapshot);
        let detail = detail_text(&snapshot);
        assert!(catalog.contains("repo"));
        assert!(catalog.contains("/tmp/repo"));
        assert!(detail.contains("Project / lib selector"));
        assert!(detail.contains("agentic-workflow"));
        assert!(detail.contains("Repo View Desktop App"));
        assert!(detail.contains("Capabilities: 1"));
    }
}
