#![allow(unexpected_cfgs)]

use anyhow::Result;

use crate::cli::view::{RepoViewItemSnapshot, RepoViewSnapshot};

pub fn catalog_text(snapshot: &RepoViewSnapshot) -> String {
    let selected = snapshot.selected.as_deref().unwrap_or_default();
    let mut out = String::new();
    out.push_str(&format!(
        "AW Repo View\n{}\n\n{} items / {} projects / {} libs\n\n",
        snapshot.repo.name,
        snapshot.repo.item_count,
        snapshot.repo.project_count,
        snapshot.repo.library_count
    ));
    for item in &snapshot.catalog {
        let marker = if item.name == selected { ">" } else { " " };
        out.push_str(&format!(
            "{marker} {} [{}]\n  {}\n",
            item.name, item.kind, item.path
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
        "{}\n{}\n{}\n\nREADME\n{}\n\n{}\n\n",
        item.readme.title,
        item.project.kind,
        item.project.path,
        item.readme.path,
        item.readme.brief
    ));
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
    use crate::cli::view::{
        layout_toggle_button_label, render_app_screenshot_png, toggled_view_layout,
        RepoViewSnapshot, APP_SCREENSHOT_HEIGHT, APP_SCREENSHOT_WIDTH,
    };
    use anyhow::Result;
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel, NO, YES};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::c_void;
    use std::ptr;
    use std::sync::Once;

    #[link(name = "AppKit", kind = "framework")]
    extern "C" {}

    type Id = *mut Object;

    const NS_UTF8_STRING_ENCODING: usize = 4;
    const NS_BACKING_STORE_BUFFERED: usize = 2;
    const NS_WINDOW_STYLE_MASK_TITLED: usize = 1 << 0;
    const NS_WINDOW_STYLE_MASK_CLOSABLE: usize = 1 << 1;
    const NS_WINDOW_STYLE_MASK_MINIATURIZABLE: usize = 1 << 2;
    const NS_WINDOW_STYLE_MASK_RESIZABLE: usize = 1 << 3;
    const NS_VIEW_WIDTH_SIZABLE: usize = 1 << 1;
    const NS_VIEW_HEIGHT_SIZABLE: usize = 1 << 4;
    const NS_VIEW_MIN_X_MARGIN: usize = 1 << 0;
    const NS_VIEW_MIN_Y_MARGIN: usize = 1 << 3;
    const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: isize = 0;
    const NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN: usize = 3;
    const NS_BEZEL_STYLE_ROUNDED: usize = 1;

    struct NativeRepoViewState {
        snapshot: RepoViewSnapshot,
        image_view: Id,
        button: Id,
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
            let _: () = msg_send![app, activateIgnoringOtherApps: YES];
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
        let title = ns_string(&format!("AW Repo View - {}", snapshot.repo.name));
        let _: () = msg_send![window, setTitle: title];
        let _: () = msg_send![window, center];

        let content_view = make_content_view(snapshot)?;
        let _: () = msg_send![window, setContentView: content_view];
        Ok(window)
    }

    unsafe fn make_content_view(snapshot: &RepoViewSnapshot) -> Result<Id> {
        let frame = rect(
            0.0,
            0.0,
            APP_SCREENSHOT_WIDTH as f64,
            APP_SCREENSHOT_HEIGHT as f64,
        );
        let root_view: Id = msg_send![class!(NSView), alloc];
        let root_view: Id = msg_send![root_view, initWithFrame: frame];
        let _: () = msg_send![root_view, setAutoresizingMask: autoresize_mask()];

        let image_view: Id = msg_send![class!(NSImageView), alloc];
        let image_view: Id = msg_send![image_view, initWithFrame: frame];
        let _: () = msg_send![image_view, setAutoresizingMask: autoresize_mask()];
        let _: () =
            msg_send![image_view, setImageScaling: NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN];
        let image = make_snapshot_image(snapshot)?;
        let _: () = msg_send![image_view, setImage: image];

        let button = make_layout_toggle_button(snapshot);
        let state = Box::into_raw(Box::new(NativeRepoViewState {
            snapshot: snapshot.clone(),
            image_view,
            button,
        }));
        let controller = make_layout_toggle_controller(state.cast::<c_void>());
        let _: () = msg_send![button, setTarget: controller];
        let _: () = msg_send![button, setAction: sel!(toggleLayout:)];

        let _: () = msg_send![root_view, addSubview: image_view];
        let _: () = msg_send![root_view, addSubview: button];
        Ok(root_view)
    }

    unsafe fn make_layout_toggle_button(snapshot: &RepoViewSnapshot) -> Id {
        let frame = rect(
            APP_SCREENSHOT_WIDTH as f64 - 218.0,
            APP_SCREENSHOT_HEIGHT as f64 - 44.0,
            190.0,
            30.0,
        );
        let button: Id = msg_send![class!(NSButton), alloc];
        let button: Id = msg_send![button, initWithFrame: frame];
        let _: () = msg_send![button, setBezelStyle: NS_BEZEL_STYLE_ROUNDED];
        let _: () = msg_send![button, setAutoresizingMask: top_right_autoresize_mask()];
        let font: Id = msg_send![class!(NSFont), systemFontOfSize: 12.0];
        let _: () = msg_send![button, setFont: font];
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
            match make_snapshot_image(&state.snapshot) {
                Ok(image) => {
                    let _: () = msg_send![state.image_view, setImage: image];
                    set_layout_toggle_button_title(state.button, &state.snapshot);
                    let _: () = msg_send![state.image_view, setNeedsDisplay: YES];
                    let _: () = msg_send![state.button, setNeedsDisplay: YES];
                }
                Err(err) => eprintln!("aw view layout toggle failed: {err:#}"),
            }
        }
    }

    unsafe fn set_layout_toggle_button_title(button: Id, snapshot: &RepoViewSnapshot) {
        let title = ns_string(layout_toggle_button_label(snapshot.layout));
        let _: () = msg_send![button, setTitle: title];
    }

    unsafe fn make_snapshot_image(snapshot: &RepoViewSnapshot) -> Result<Id> {
        let png = render_app_screenshot_png(snapshot)?;
        let data: Id = msg_send![
            class!(NSData),
            dataWithBytes: png.as_ptr()
            length: png.len()
        ];
        let image: Id = msg_send![class!(NSImage), alloc];
        let image: Id = msg_send![image, initWithData: data];
        if image.is_null() {
            anyhow::bail!("failed to create NSImage from app screenshot PNG");
        }
        Ok(image)
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

    fn autoresize_mask() -> usize {
        NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
    }

    fn top_right_autoresize_mask() -> usize {
        NS_VIEW_MIN_X_MARGIN | NS_VIEW_MIN_Y_MARGIN
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
                cap_path: Some("projects/agentic-workflow/README.md".to_string()),
            }],
            selected: Some("agentic-workflow".to_string()),
            items: vec![RepoViewItemSnapshot {
                project: crate::cli::view::ProjectViewProject {
                    name: "agentic-workflow".to_string(),
                    aliases: vec!["aw".to_string()],
                    kind: "project".to_string(),
                    path: "projects/agentic-workflow".to_string(),
                    td_path: "projects/agentic-workflow/tech-design".to_string(),
                    cap_path: "projects/agentic-workflow/README.md".to_string(),
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
        assert!(catalog.contains("agentic-workflow"));
        assert!(catalog.contains("projects/agentic-workflow"));
        assert!(detail.contains("Repo View Desktop App"));
        assert!(detail.contains("Capabilities: 1"));
    }
}
