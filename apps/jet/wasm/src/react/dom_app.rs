// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
// CODEGEN-BEGIN
//! DOM event loop for Jet WASM compatibility builds.
//!
//! Canvas remains Jet's renderer experiment target. Real product
//! frontends also need form controls, normal text selection, and CSS
//! while the compiler/runtime subset catches up. This module keeps the
//! same Rust/WASM ownership model as `canvas_app`: the Element tree,
//! state, effects, and event handlers live in WASM; JavaScript stays
//! as the thin boot + host-capability bridge.

#![cfg(feature = "dom-app")]

use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::{Element, Props};

use super::{mount, MountHandle};

/// Mount `component` on a DOM element and keep it rerendered.
///
/// The current implementation intentionally does full subtree
/// replacement. That is coarse but deterministic, and it gives
/// compatibility builds the browser-native controls they need without
/// moving app logic into JavaScript.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn run(root_id: &str, component: crate::Component) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let root = document
        .get_element_by_id(root_id)
        .ok_or_else(|| JsValue::from_str("DOM mount element not found"))?;

    let root = Rc::new(root);
    let handle = Rc::new(mount(component));
    render_current(&root, &handle)?;
    install_flush_loop(window, root, handle)?;
    Ok(())
}

fn install_flush_loop(
    window: web_sys::Window,
    root: Rc<web_sys::Element>,
    handle: Rc<MountHandle>,
) -> Result<(), JsValue> {
    let tick: Rc<std::cell::RefCell<Option<Closure<dyn FnMut()>>>> =
        Rc::new(std::cell::RefCell::new(None));
    let tick_for_closure = tick.clone();
    let window_for_closure = window.clone();
    *tick_for_closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if handle.flush() {
            let _ = render_current(&root, &handle);
        }
        if let Some(cb) = tick.borrow().as_ref() {
            let _ = window_for_closure.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut()>));

    if let Some(cb) = tick_for_closure.borrow().as_ref() {
        window.request_animation_frame(cb.as_ref().unchecked_ref())?;
    }
    Ok(())
}

fn render_current(root: &web_sys::Element, handle: &Rc<MountHandle>) -> Result<(), JsValue> {
    let document = root
        .owner_document()
        .ok_or_else(|| JsValue::from_str("mount root has no owner document"))?;
    root.set_inner_html("");
    let node = build_node(&document, &handle.snapshot(), root, handle)?;
    root.append_child(&node)?;
    Ok(())
}

fn build_node(
    document: &web_sys::Document,
    element: &Element,
    root: &web_sys::Element,
    handle: &Rc<MountHandle>,
) -> Result<web_sys::Node, JsValue> {
    match element {
        Element::Empty => Ok(document.create_text_node("").into()),
        Element::Text(text) => Ok(document.create_text_node(text).into()),
        Element::Fragment(children) => {
            let fragment = document.create_document_fragment();
            for child in children {
                fragment.append_child(&build_node(document, child, root, handle)?)?;
            }
            Ok(fragment.into())
        }
        Element::Component(component) => {
            let rendered = (component.render)(&component.props);
            build_node(document, &rendered, root, handle)
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } => {
            let tag = dom_tag(tag);
            let node = document.create_element(tag)?;
            apply_props(&node, props)?;
            install_events(&node, props, root, handle)?;
            for child in children {
                node.append_child(&build_node(document, child, root, handle)?)?;
            }
            Ok(node.into())
        }
    }
}

fn dom_tag(tag: &str) -> &str {
    match tag {
        "Fragment" | "__fragment" => "div",
        "input" | "textarea" | "button" | "span" | "main" | "aside" | "section" | "article"
        | "nav" | "form" | "label" | "h1" | "h2" | "h3" | "p" | "svg" | "path" => tag,
        _ => "div",
    }
}

fn apply_props(node: &web_sys::Element, props: &Props) -> Result<(), JsValue> {
    if let Some(id) = &props.id {
        node.set_attribute("id", id)?;
    }
    if let Some(class_name) = &props.class_name {
        node.set_attribute("class", class_name)?;
    }
    if let Some(style) = &props.style {
        node.set_attribute("style", style)?;
    }
    if let Some(placeholder) = &props.placeholder {
        node.set_attribute("placeholder", placeholder)?;
    }
    // @spec .aw/tech-design/projects/jet/specs/4072.md#logic
    if let Some(input_type) = &props.input_type {
        node.set_attribute("type", input_type)?;
    }
    if let Some(aria_label) = &props.aria_label {
        node.set_attribute("aria-label", aria_label)?;
    }
    if let Some(html_for) = &props.html_for {
        node.set_attribute("for", html_for)?;
    }
    if props.disabled {
        node.set_attribute("disabled", "true")?;
    }
    if let Some(checked) = props.checked {
        if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
            input.set_checked(checked);
        }
        if checked {
            node.set_attribute("checked", "true")?;
        } else {
            node.remove_attribute("checked")?;
        }
    }
    if let Some(value) = &props.value {
        if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
            input.set_value(value);
        } else if let Some(textarea) = node.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            textarea.set_value(value);
            node.set_text_content(Some(value));
        } else {
            node.set_attribute("value", value)?;
        }
    }
    Ok(())
}

fn event_target_value(e: web_sys::Event) -> String {
    let Some(target) = e.target() else {
        return String::new();
    };
    if let Ok(input) = target.clone().dyn_into::<web_sys::HtmlInputElement>() {
        return input.value();
    }
    if let Ok(textarea) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
        return textarea.value();
    }
    String::new()
}

fn event_target_checked(e: web_sys::Event) -> bool {
    let Some(target) = e.target() else {
        return false;
    };
    if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
        return input.checked();
    }
    false
}

fn install_events(
    node: &web_sys::Element,
    props: &Props,
    root: &web_sys::Element,
    handle: &Rc<MountHandle>,
) -> Result<(), JsValue> {
    if let Some(on_click) = props.on_click.clone() {
        let root = root.clone();
        let handle = handle.clone();
        let click_cb = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            on_click.call(());
            if handle.flush() {
                let _ = render_current(&root, &handle);
            }
        }) as Box<dyn FnMut(_)>);
        node.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref())?;
        click_cb.forget();
    }

    if let Some(on_change) = props.on_change.clone() {
        let root = root.clone();
        let handle = handle.clone();
        let input_cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let value = event_target_value(e);
            on_change.call(value);
            if handle.flush() {
                let _ = render_current(&root, &handle);
            }
        }) as Box<dyn FnMut(_)>);
        node.add_event_listener_with_callback("input", input_cb.as_ref().unchecked_ref())?;
        input_cb.forget();
    }

    if let Some(on_checked_change) = props.on_checked_change.clone() {
        let root = root.clone();
        let handle = handle.clone();
        let change_cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let checked = event_target_checked(e);
            on_checked_change.call(checked);
            if handle.flush() {
                let _ = render_current(&root, &handle);
            }
        }) as Box<dyn FnMut(_)>);
        node.add_event_listener_with_callback("change", change_cb.as_ref().unchecked_ref())?;
        change_cb.forget();
    }

    Ok(())
}
// CODEGEN-END
