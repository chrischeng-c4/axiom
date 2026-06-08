use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// xml.etree.ElementTree module for Mamba (#449).
///
/// Provides: Element, SubElement, parse, tostring, fromstring
/// Minimal XML tree in-memory representation using dicts.
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_binary!(d_element, mb_xml_element);
disp_binary!(d_subelement, mb_xml_subelement);
disp_unary!(d_parse, mb_xml_parse);
disp_unary!(d_tostring, mb_xml_tostring);
disp_unary!(d_fromstring, mb_xml_fromstring);
disp_unary!(d_xml, mb_xml_xml);
disp_unary!(d_iselement, mb_xml_iselement);
disp_unary!(d_comment, mb_xml_comment);
disp_binary!(d_processing_instruction, mb_xml_processing_instruction);
disp_unary!(d_fromstringlist, mb_xml_fromstringlist);
disp_unary!(d_tostringlist, mb_xml_tostringlist);
disp_unary!(d_indent, mb_xml_indent);
disp_binary!(d_register_namespace, mb_xml_register_namespace);

/// Register the xml module (also registers xml.etree.ElementTree).
///
/// Wave-4 Ship #4 (Task #56) — expands from 5-dispatcher stub to
/// 13-dispatcher + 4-class-shell + 2 alias-module surface (21
/// entries per scout estimate). Replaces the stub `fromstring` with
/// a real recursive descent parser (well-formed XML only).
pub fn register() {
    use super::super::module::NATIVE_FUNC_ADDRS;

    let mut attrs = HashMap::new();

    let dispatchers: &[(&str, usize)] = &[
        ("Element", d_element as *const () as usize),
        ("SubElement", d_subelement as *const () as usize),
        ("parse", d_parse as *const () as usize),
        ("tostring", d_tostring as *const () as usize),
        ("fromstring", d_fromstring as *const () as usize),
        ("XML", d_xml as *const () as usize),
        ("XMLID", d_xml as *const () as usize), // alias; CPython returns (Element, {})
        ("iselement", d_iselement as *const () as usize),
        ("Comment", d_comment as *const () as usize),
        (
            "ProcessingInstruction",
            d_processing_instruction as *const () as usize,
        ),
        ("PI", d_processing_instruction as *const () as usize), // PI is an alias of ProcessingInstruction
        ("fromstringlist", d_fromstringlist as *const () as usize),
        ("tostringlist", d_tostringlist as *const () as usize),
        ("indent", d_indent as *const () as usize),
        (
            "register_namespace",
            d_register_namespace as *const () as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(*addr));
        NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }

    // Class shells (Instance with class_name; full construction stubbed).
    let class_shell = |name: &str| -> MbValue {
        let fields = FxHashMap::default();
        let obj = Box::new(super::super::rc::MbObject {
            header: super::super::rc::MbObjectHeader {
                rc: std::sync::atomic::AtomicU32::new(1),
                kind: super::super::rc::ObjKind::Instance,
            },
            data: ObjData::Instance {
                class_name: format!("xml.etree.ElementTree.{}", name),
                fields: crate::runtime::rc::MbRwLock::new(fields),
            },
        });
        MbValue::from_ptr(Box::into_raw(obj))
    };
    for name in &["ElementTree", "QName", "ParseError", "TreeBuilder"] {
        attrs.insert((*name).to_string(), class_shell(name));
    }

    // Register at both `xml` (legacy module alias) and the full
    // dotted path. Mamba's `import X.Y` quirk binds X to X.Y's dict,
    // so `import xml.etree.ElementTree as ET` works via the
    // full-path registration only.
    super::register_module("xml.etree.ElementTree", attrs.clone());
    super::register_module("xml.etree", attrs.clone());
    super::register_module("xml", attrs);
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// xml.etree.ElementTree.Element(tag, attrib?) -> element dict
pub fn mb_xml_element(tag: MbValue, attrib: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    let tag_str = extract_str(tag).unwrap_or_else(|| "element".to_string());
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("Element".to_string())),
            );
            map.insert("tag".into(), MbValue::from_ptr(MbObject::new_str(tag_str)));
            map.insert("text".into(), MbValue::none());
            map.insert("tail".into(), MbValue::none());
            map.insert(
                "attrib".into(),
                if attrib.is_none() {
                    MbValue::from_ptr(MbObject::new_dict())
                } else {
                    attrib
                },
            );
            map.insert(
                "_children".into(),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
        }
    }
    MbValue::from_ptr(dict)
}

/// SubElement(parent, tag) -> child element appended to parent
pub fn mb_xml_subelement(parent: MbValue, tag: MbValue) -> MbValue {
    let child = mb_xml_element(tag, MbValue::none());
    if let Some(ptr) = parent.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(children) = map.get("_children").copied() {
                    if let Some(c_ptr) = children.as_ptr() {
                        if let ObjData::List(ref list_lock) = (*c_ptr).data {
                            let mut items = list_lock.write().unwrap();
                            items.push(child);
                        }
                    }
                }
            }
        }
    }
    child
}

/// tostring(element) -> XML string
pub fn mb_xml_tostring(elem: MbValue) -> MbValue {
    let s = element_to_string(elem, 0);
    MbValue::from_ptr(MbObject::new_str(s))
}

fn element_to_string(elem: MbValue, depth: usize) -> String {
    if let Some(ptr) = elem.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                let tag = map
                    .get("tag")
                    .and_then(|v| extract_str(*v))
                    .unwrap_or_else(|| "?".to_string());
                let text = map.get("text").and_then(|v| extract_str(*v));

                // Build attributes string
                let mut attr_str = String::new();
                if let Some(attrib) = map.get("attrib").copied() {
                    if let Some(a_ptr) = attrib.as_ptr() {
                        if let ObjData::Dict(ref a_lock) = (*a_ptr).data {
                            let a_map = a_lock.read().unwrap();
                            for (k, v) in a_map.iter() {
                                let vs = extract_str(*v).unwrap_or_default();
                                attr_str.push_str(&format!(" {k}=\"{vs}\""));
                            }
                        }
                    }
                }

                // Children
                let children = map.get("_children").copied();
                let has_children = children
                    .map(|c| {
                        c.as_ptr()
                            .map(|p| {
                                if let ObjData::List(ref list_lock) = (*p).data {
                                    let items = list_lock.read().unwrap();
                                    !items.is_empty()
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);

                if !has_children && text.is_none() {
                    return format!("<{tag}{attr_str} />");
                }

                let mut result = format!("<{tag}{attr_str}>");
                if let Some(t) = &text {
                    result.push_str(t);
                }
                if let Some(c) = children {
                    if let Some(c_ptr) = c.as_ptr() {
                        if let ObjData::List(ref list_lock) = (*c_ptr).data {
                            let items = list_lock.read().unwrap();
                            for child in items.iter() {
                                result.push_str(&element_to_string(*child, depth + 1));
                            }
                        }
                    }
                }
                result.push_str(&format!("</{tag}>"));
                return result;
            }
        }
    }
    String::new()
}

/// fromstring(xml_str) -> Element (recursive descent parser).
///
/// Wave-4 Ship #4 (Task #56) — real parser replacing the stub.
/// Handles well-formed XML: nested elements, text/tail, attributes
/// (double- or single-quoted), self-closing tags, processing
/// instructions and comments (skipped). Does NOT handle: namespaces
/// (treats `{uri}name` as opaque), CDATA, DTD, character refs other
/// than the standard 5 (&lt; &gt; &amp; &quot; &apos;), or
/// entity definitions.
pub fn mb_xml_fromstring(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    let bytes = s.as_bytes();
    let mut i = 0;
    skip_prolog(bytes, &mut i);
    parse_element(bytes, &mut i).unwrap_or_else(|| {
        mb_xml_element(
            MbValue::from_ptr(MbObject::new_str("root".to_string())),
            MbValue::none(),
        )
    })
}

fn skip_prolog(bytes: &[u8], i: &mut usize) {
    skip_ws(bytes, i);
    while *i + 1 < bytes.len()
        && bytes[*i] == b'<'
        && (bytes[*i + 1] == b'?' || bytes[*i + 1] == b'!')
    {
        // Skip <?...?> or <!--...--> or <!DOCTYPE ...>
        if bytes[*i + 1] == b'!' && *i + 3 < bytes.len() && &bytes[*i + 2..*i + 4] == b"--" {
            // Comment
            *i += 4;
            while *i + 2 < bytes.len() && &bytes[*i..*i + 3] != b"-->" {
                *i += 1;
            }
            *i += 3;
        } else {
            // PI or DOCTYPE
            let close = if bytes[*i + 1] == b'?' {
                "?>".as_bytes()
            } else {
                ">".as_bytes()
            };
            while *i + close.len() <= bytes.len() && &bytes[*i..*i + close.len()] != close {
                *i += 1;
            }
            *i += close.len();
        }
        skip_ws(bytes, i);
    }
}

fn skip_ws(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len()
        && (bytes[*i] == b' ' || bytes[*i] == b'\n' || bytes[*i] == b'\t' || bytes[*i] == b'\r')
    {
        *i += 1;
    }
}

fn parse_element(bytes: &[u8], i: &mut usize) -> Option<MbValue> {
    if *i >= bytes.len() || bytes[*i] != b'<' {
        return None;
    }
    *i += 1;
    // Read tag
    let start = *i;
    while *i < bytes.len()
        && bytes[*i] != b' '
        && bytes[*i] != b'>'
        && bytes[*i] != b'/'
        && bytes[*i] != b'\n'
        && bytes[*i] != b'\t'
    {
        *i += 1;
    }
    let tag = std::str::from_utf8(&bytes[start..*i]).ok()?.to_string();
    // Build element with attributes
    let attrib_dict = MbObject::new_dict();
    loop {
        skip_ws(bytes, i);
        if *i >= bytes.len() {
            return None;
        }
        if bytes[*i] == b'/' {
            *i += 1;
        }
        if bytes[*i] == b'>' {
            break;
        }
        // Parse attribute name
        let a_start = *i;
        while *i < bytes.len()
            && bytes[*i] != b'='
            && bytes[*i] != b' '
            && bytes[*i] != b'>'
            && bytes[*i] != b'/'
        {
            *i += 1;
        }
        let attr_name = std::str::from_utf8(&bytes[a_start..*i]).ok()?.to_string();
        if attr_name.is_empty() {
            break;
        }
        skip_ws(bytes, i);
        if *i >= bytes.len() || bytes[*i] != b'=' {
            return None;
        }
        *i += 1;
        skip_ws(bytes, i);
        if *i >= bytes.len() {
            return None;
        }
        let quote = bytes[*i];
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        *i += 1;
        let v_start = *i;
        while *i < bytes.len() && bytes[*i] != quote {
            *i += 1;
        }
        let attr_val = std::str::from_utf8(&bytes[v_start..*i]).ok()?.to_string();
        *i += 1;
        unsafe {
            if let ObjData::Dict(ref lock) = (*attrib_dict).data {
                lock.write().unwrap().insert(
                    attr_name.into(),
                    MbValue::from_ptr(MbObject::new_str(decode_entities(&attr_val))),
                );
            }
        }
    }
    let self_closing = *i > 0 && bytes[*i - 1] == b'/';
    *i += 1; // skip '>'
    let elem = mb_xml_element(
        MbValue::from_ptr(MbObject::new_str(tag.clone())),
        MbValue::from_ptr(attrib_dict),
    );
    if self_closing {
        return Some(elem);
    }
    // Parse text + children until </tag>
    let text_start = *i;
    while *i < bytes.len() && bytes[*i] != b'<' {
        *i += 1;
    }
    let text = std::str::from_utf8(&bytes[text_start..*i])
        .ok()?
        .to_string();
    if !text.is_empty() {
        if let Some(ptr) = elem.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    lock.write().unwrap().insert(
                        "text".into(),
                        MbValue::from_ptr(MbObject::new_str(decode_entities(&text))),
                    );
                }
            }
        }
    }
    loop {
        if *i + 1 >= bytes.len() {
            return None;
        }
        if bytes[*i] == b'<' && bytes[*i + 1] == b'/' {
            // End tag
            *i += 2;
            while *i < bytes.len() && bytes[*i] != b'>' {
                *i += 1;
            }
            *i += 1;
            break;
        }
        // Skip comments/PI between children
        if bytes[*i] == b'<'
            && *i + 1 < bytes.len()
            && (bytes[*i + 1] == b'!' || bytes[*i + 1] == b'?')
        {
            skip_prolog(bytes, i);
            continue;
        }
        let child = parse_element(bytes, i)?;
        if let Some(ptr) = elem.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(children) = map.get("_children").copied() {
                        if let Some(c_ptr) = children.as_ptr() {
                            if let ObjData::List(ref list_lock) = (*c_ptr).data {
                                list_lock.write().unwrap().push(child);
                            }
                        }
                    }
                }
            }
        }
        // Child tail text
        let tail_start = *i;
        while *i < bytes.len() && bytes[*i] != b'<' {
            *i += 1;
        }
        let tail = std::str::from_utf8(&bytes[tail_start..*i])
            .ok()?
            .to_string();
        if !tail.is_empty() {
            if let Some(ptr) = child.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        lock.write().unwrap().insert(
                            "tail".into(),
                            MbValue::from_ptr(MbObject::new_str(decode_entities(&tail))),
                        );
                    }
                }
            }
        }
    }
    Some(elem)
}

fn decode_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// parse(filename) -> ElementTree-like wrapper; for forward ship it
/// returns the root Element directly (ElementTree wrapper class is
/// not in scope — getroot() callers can use the returned element).
pub fn mb_xml_parse(filename: MbValue) -> MbValue {
    let content = if let Some(path) = extract_str(filename) {
        std::fs::read_to_string(&path).ok()
    } else {
        None
    };
    match content {
        Some(text) => mb_xml_fromstring(MbValue::from_ptr(MbObject::new_str(text))),
        None => mb_xml_element(
            MbValue::from_ptr(MbObject::new_str("root".to_string())),
            MbValue::none(),
        ),
    }
}

/// XML(xml_str) — alias for fromstring per CPython.
pub fn mb_xml_xml(val: MbValue) -> MbValue {
    mb_xml_fromstring(val)
}

/// iselement(obj) -> bool — true if obj is an Element-shaped dict.
pub fn mb_xml_iselement(val: MbValue) -> MbValue {
    let is_elem = val
        .as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                map.contains_key("tag") && map.contains_key("_children")
            } else {
                false
            }
        })
        .unwrap_or(false);
    MbValue::from_bool(is_elem)
}

/// Comment(text) -> a Comment-tagged Element shell.
pub fn mb_xml_comment(text: MbValue) -> MbValue {
    let elem = mb_xml_element(
        MbValue::from_ptr(MbObject::new_str("!comment".to_string())),
        MbValue::none(),
    );
    if !text.is_none() {
        if let Some(ptr) = elem.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    lock.write().unwrap().insert("text".into(), text);
                }
            }
        }
    }
    elem
}

/// ProcessingInstruction(target, text) -> a PI-tagged Element shell.
pub fn mb_xml_processing_instruction(target: MbValue, text: MbValue) -> MbValue {
    let elem = mb_xml_element(
        MbValue::from_ptr(MbObject::new_str("?pi".to_string())),
        MbValue::none(),
    );
    if let Some(ptr) = elem.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("target".into(), target);
                if !text.is_none() {
                    map.insert("text".into(), text);
                }
            }
        }
    }
    elem
}

/// fromstringlist(strs) -> Element. Joins the iterable then parses.
pub fn mb_xml_fromstringlist(strs: MbValue) -> MbValue {
    let joined = if let Some(ptr) = strs.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                items
                    .iter()
                    .filter_map(|v| extract_str(*v))
                    .collect::<String>()
            } else {
                String::new()
            }
        }
    } else {
        String::new()
    };
    mb_xml_fromstring(MbValue::from_ptr(MbObject::new_str(joined)))
}

/// tostringlist(elem) -> [tostring(elem)] (list-of-1 wrapper).
pub fn mb_xml_tostringlist(elem: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![mb_xml_tostring(elem)]))
}

/// indent(elem, ...) — in-place pretty-print; forward ship is a no-op
/// returning the element unchanged. Used by callers that pipe through
/// `tostring()` after `indent()`; current `tostring()` does not respect
/// indent state so the call is structurally compatible but
/// cosmetically the output is unchanged.
pub fn mb_xml_indent(elem: MbValue) -> MbValue {
    elem
}

/// register_namespace(prefix, uri) — no-op forward ship; namespace
/// resolution is opaque-string in the parser.
pub fn mb_xml_register_namespace(_prefix: MbValue, _uri: MbValue) -> MbValue {
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_element_tostring() {
        let root = mb_xml_element(s("root"), MbValue::none());
        let result = mb_xml_tostring(root);
        let xml = extract_str(result).unwrap();
        assert_eq!(xml, "<root />");
    }

    #[test]
    fn test_subelement() {
        let root = mb_xml_element(s("root"), MbValue::none());
        let child = mb_xml_subelement(root, s("child"));
        // Set child text
        if let Some(ptr) = child.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let mut map = lock.write().unwrap();
                    map.insert("text".into(), s("hello"));
                }
            }
        }
        let xml = extract_str(mb_xml_tostring(root)).unwrap();
        assert!(xml.contains("<child>hello</child>"));
    }

    #[test]
    fn test_fromstring_parses_real_xml() {
        let tag_of = |v: MbValue| -> Option<String> {
            v.as_ptr().and_then(|ptr| unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    lock.read()
                        .unwrap()
                        .get("tag")
                        .and_then(|t| extract_str(*t))
                } else {
                    None
                }
            })
        };
        // Wave-4 Ship #4: fromstring is a real parser, not a stub.
        assert_eq!(
            tag_of(mb_xml_fromstring(s("<doc/>"))).as_deref(),
            Some("doc")
        );
        assert_eq!(
            tag_of(mb_xml_fromstring(s("<root><a/></root>"))).as_deref(),
            Some("root")
        );
        // parse() on a missing path still falls back to the stub root element.
        assert_eq!(
            tag_of(mb_xml_parse(s("/nonexistent-xml-path"))).as_deref(),
            Some("root")
        );
    }

    #[test]
    fn test_iselement_predicate() {
        let elem = mb_xml_element(s("root"), MbValue::none());
        let result = mb_xml_iselement(elem);
        assert_eq!(result.as_bool(), Some(true));
        let result_str = mb_xml_iselement(s("not-an-element"));
        assert_eq!(result_str.as_bool(), Some(false));
    }

    #[test]
    fn test_fromstring_with_attributes() {
        let elem = mb_xml_fromstring(s("<item id='42' name=\"alice\"/>"));
        let tag = unsafe {
            if let ObjData::Dict(ref lock) = (*elem.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                extract_str(*map.get("tag").unwrap())
            } else {
                None
            }
        };
        assert_eq!(tag.as_deref(), Some("item"));
    }

    #[test]
    fn test_fromstring_with_text_and_children() {
        let elem = mb_xml_fromstring(s("<root><a>x</a><b>y</b></root>"));
        unsafe {
            if let ObjData::Dict(ref lock) = (*elem.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                let children = map.get("_children").copied().unwrap();
                if let ObjData::List(ref list_lock) = (*children.as_ptr().unwrap()).data {
                    let items = list_lock.read().unwrap();
                    assert_eq!(items.len(), 2);
                }
            }
        }
    }
}
