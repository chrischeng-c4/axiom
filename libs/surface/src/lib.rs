// HANDWRITE-BEGIN gap="missing-generator:surface-core" tracker="pending-tracker" reason="Shared renderer-neutral Element/Props/callback and serializable surface snapshot core extracted from jet-wasm."
//! Renderer-neutral UI surface primitives.
//!
//! `cclab-surface` is deliberately below any renderer or framework runtime. It
//! owns the UI element tree shape that Jet WASM TSX/Vue/etc. adapters produce,
//! plus a serializable snapshot form that native desktop readers, renderers, tests,
//! and parity comparators can inspect without a browser or toolkit-private tree.

use std::rc::Rc;

use serde::{Deserialize, Serialize};

/// A rendered element tree shared by framework runtimes.
#[derive(Clone)]
pub enum Element {
    /// An intrinsic renderer-neutral node: kind/tag + props + children.
    Intrinsic {
        tag: &'static str,
        props: Props,
        children: Vec<Element>,
    },
    /// A text leaf.
    Text(String),
    /// A component invocation; framework runtimes expand this before rendering.
    Component(Component),
    /// An empty/noop element. Used for conditional null/undefined in JSX.
    Empty,
    /// Transparent container for a dynamic list of elements.
    Fragment(Vec<Element>),
}

impl Element {
    pub fn intrinsic(tag: &'static str, props: Props, children: Vec<Element>) -> Self {
        Self::Intrinsic {
            tag,
            props,
            children,
        }
    }

    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    pub fn from_number<N: std::fmt::Display>(n: N) -> Self {
        Self::Text(n.to_string())
    }

    /// Depth-first collect an `on_click` callback by semantic id.
    pub fn find_on_click(&self, target_id: &str) -> Option<Callback<()>> {
        match self {
            Element::Intrinsic {
                props, children, ..
            } => {
                if props.id.as_deref() == Some(target_id) {
                    return props.on_click.clone();
                }
                children.iter().find_map(|c| c.find_on_click(target_id))
            }
            Element::Fragment(children) => children.iter().find_map(|c| c.find_on_click(target_id)),
            Element::Component(_) | Element::Text(_) | Element::Empty => None,
        }
    }

    /// Concatenate all text descendants in order.
    pub fn text_content(&self) -> String {
        match self {
            Element::Text(s) => s.clone(),
            Element::Intrinsic { children, .. } | Element::Fragment(children) => {
                children.iter().map(Element::text_content).collect()
            }
            Element::Component(_) | Element::Empty => String::new(),
        }
    }

    /// Capture a deterministic semantic surface snapshot.
    pub fn surface_snapshot(&self) -> SurfaceSnapshot {
        SurfaceSnapshot::from_element(self)
    }
}

/// Component = render function + typed props erased behind `Any`.
#[derive(Clone)]
pub struct Component {
    pub name: &'static str,
    pub render: ComponentFn,
    pub props: Rc<dyn std::any::Any>,
}

pub type ComponentFn = fn(&Rc<dyn std::any::Any>) -> Element;

/// Host props shared by framework runtimes and renderers.
#[derive(Clone, Default, Debug)]
pub struct Props {
    pub class_name: Option<String>,
    pub style: Option<String>,
    pub on_click: Option<Callback<()>>,
    pub on_change: Option<Callback<String>>,
    pub on_checked_change: Option<Callback<bool>>,
    pub id: Option<String>,
    pub value: Option<String>,
    pub input_type: Option<String>,
    pub placeholder: Option<String>,
    pub checked: Option<bool>,
    pub aria_label: Option<String>,
    pub html_for: Option<String>,
    pub disabled: bool,
}

/// Event callback typed by payload.
#[derive(Clone)]
pub struct Callback<P: Clone>(Rc<dyn Fn(P)>);

impl<P: Clone> std::fmt::Debug for Callback<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<callback>")
    }
}

impl<P: Clone> Callback<P> {
    pub fn new<F: Fn(P) + 'static>(f: F) -> Self {
        Self(Rc::new(f))
    }

    pub fn call(&self, payload: P) {
        (self.0)(payload);
    }
}

/// Serializable snapshot of a rendered surface tree.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SurfaceSnapshot {
    pub schema_version: u32,
    pub nodes: Vec<SurfaceNode>,
}

impl SurfaceSnapshot {
    pub const SCHEMA_VERSION: u32 = 1;

    pub fn from_element(root: &Element) -> Self {
        let mut snapshot = Self {
            schema_version: Self::SCHEMA_VERSION,
            nodes: Vec::new(),
        };
        push_snapshot_nodes(root, None, "root".to_string(), &mut snapshot);
        snapshot
    }

    pub fn get(&self, node_id: &str) -> Option<&SurfaceNode> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }

    pub fn find_by_semantic_id(&self, semantic_id: &str) -> Option<&SurfaceNode> {
        self.nodes
            .iter()
            .find(|node| node.semantic_id == semantic_id)
    }

    pub fn find_by_role<'a>(&'a self, role: &str) -> Vec<&'a SurfaceNode> {
        self.nodes
            .iter()
            .filter(|node| node.role.as_deref() == Some(role))
            .collect()
    }

    pub fn text_content(&self) -> String {
        self.nodes
            .iter()
            .filter_map(|node| node.text.as_deref())
            .collect()
    }

    pub fn set_bounds(&mut self, node_id: &str, bounds: SurfaceRect) -> bool {
        let Some(node) = self.nodes.iter_mut().find(|node| node.node_id == node_id) else {
            return false;
        };
        node.bounds = Some(bounds);
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceNode {
    /// Stable structural path inside the rendered tree.
    pub node_id: String,
    /// Comparator/test alignment key. Uses `props.id` when available,
    /// otherwise falls back to the structural path.
    pub semantic_id: String,
    pub parent_id: Option<String>,
    pub kind: SurfaceNodeKind,
    pub tag: Option<String>,
    pub component: Option<String>,
    pub role: Option<String>,
    pub name: Option<String>,
    pub text: Option<String>,
    pub props: SurfaceProps,
    pub bounds: Option<SurfaceRect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceNodeKind {
    Element,
    Text,
    Component,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SurfaceProps {
    pub id: Option<String>,
    pub class_name: Option<String>,
    pub style: Option<String>,
    pub value: Option<String>,
    pub input_type: Option<String>,
    pub placeholder: Option<String>,
    pub checked: Option<bool>,
    pub aria_label: Option<String>,
    pub html_for: Option<String>,
    pub disabled: bool,
    pub has_on_click: bool,
    pub has_on_change: bool,
    pub has_on_checked_change: bool,
}

impl From<&Props> for SurfaceProps {
    fn from(props: &Props) -> Self {
        Self {
            id: props.id.clone(),
            class_name: props.class_name.clone(),
            style: props.style.clone(),
            value: props.value.clone(),
            input_type: props.input_type.clone(),
            placeholder: props.placeholder.clone(),
            checked: props.checked,
            aria_label: props.aria_label.clone(),
            html_for: props.html_for.clone(),
            disabled: props.disabled,
            has_on_click: props.on_click.is_some(),
            has_on_change: props.on_change.is_some(),
            has_on_checked_change: props.on_checked_change.is_some(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurfaceRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

fn push_snapshot_nodes(
    element: &Element,
    parent_id: Option<String>,
    node_id: String,
    snapshot: &mut SurfaceSnapshot,
) {
    match element {
        Element::Intrinsic {
            tag,
            props,
            children,
        } => {
            let semantic_id = props.id.clone().unwrap_or_else(|| node_id.clone());
            let name = accessible_name(*tag, props, children);
            snapshot.nodes.push(SurfaceNode {
                node_id: node_id.clone(),
                semantic_id,
                parent_id,
                kind: SurfaceNodeKind::Element,
                tag: Some((*tag).to_string()),
                component: None,
                role: role_for(*tag, props),
                name,
                text: None,
                props: SurfaceProps::from(props),
                bounds: None,
            });
            for (idx, child) in children.iter().enumerate() {
                push_snapshot_nodes(
                    child,
                    Some(node_id.clone()),
                    format!("{node_id}/{idx}"),
                    snapshot,
                );
            }
        }
        Element::Text(text) => {
            snapshot.nodes.push(SurfaceNode {
                node_id: node_id.clone(),
                semantic_id: node_id,
                parent_id,
                kind: SurfaceNodeKind::Text,
                tag: None,
                component: None,
                role: None,
                name: None,
                text: Some(text.clone()),
                props: SurfaceProps::default(),
                bounds: None,
            });
        }
        Element::Component(component) => {
            snapshot.nodes.push(SurfaceNode {
                node_id: node_id.clone(),
                semantic_id: node_id,
                parent_id,
                kind: SurfaceNodeKind::Component,
                tag: None,
                component: Some(component.name.to_string()),
                role: None,
                name: Some(component.name.to_string()),
                text: None,
                props: SurfaceProps::default(),
                bounds: None,
            });
        }
        Element::Fragment(children) => {
            for (idx, child) in children.iter().enumerate() {
                push_snapshot_nodes(
                    child,
                    parent_id.clone(),
                    format!("{node_id}/fragment/{idx}"),
                    snapshot,
                );
            }
        }
        Element::Empty => {}
    }
}

fn role_for(tag: &str, props: &Props) -> Option<String> {
    let role = match tag {
        "button" => "button",
        "input" if props.input_type.as_deref() == Some("checkbox") => "checkbox",
        "input" | "textarea" => "textbox",
        "label" => "label",
        "main" => "main",
        "nav" => "navigation",
        "table" => "table",
        "tr" => "row",
        "td" => "cell",
        "th" => "columnheader",
        "ul" | "ol" => "list",
        "li" => "listitem",
        _ => return None,
    };
    Some(role.to_string())
}

fn accessible_name(tag: &str, props: &Props, children: &[Element]) -> Option<String> {
    if let Some(label) = props.aria_label.as_ref().filter(|s| !s.is_empty()) {
        return Some(label.clone());
    }
    match tag {
        "button" | "label" | "td" | "th" => {
            let text = children
                .iter()
                .map(Element::text_content)
                .collect::<String>();
            let text = text.trim();
            (!text.is_empty()).then(|| text.to_string())
        }
        "input" | "textarea" => props
            .value
            .as_ref()
            .filter(|s| !s.is_empty())
            .or_else(|| props.placeholder.as_ref().filter(|s| !s.is_empty()))
            .cloned(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_captures_semantic_surface_tree() {
        let surface = Element::intrinsic(
            "main",
            Props {
                id: Some("app".to_string()),
                ..Default::default()
            },
            vec![Element::intrinsic(
                "button",
                Props {
                    id: Some("save".to_string()),
                    on_click: Some(Callback::new(|_| {})),
                    ..Default::default()
                },
                vec![Element::text("Save")],
            )],
        )
        .surface_snapshot();

        assert_eq!(surface.schema_version, SurfaceSnapshot::SCHEMA_VERSION);
        assert_eq!(surface.text_content(), "Save");

        let button = surface.find_by_semantic_id("save").unwrap();
        assert_eq!(button.role.as_deref(), Some("button"));
        assert_eq!(button.name.as_deref(), Some("Save"));
        assert!(button.props.has_on_click);
        assert_eq!(surface.find_by_role("button"), vec![button]);
    }
}
// HANDWRITE-END
