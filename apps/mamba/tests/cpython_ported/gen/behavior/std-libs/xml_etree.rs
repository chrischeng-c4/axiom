use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/xml_etree/element_constructor_and_subelement.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_element_constructor_and_subelement() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "element_constructor_and_subelement"
# subject = "xml.etree.ElementTree.SubElement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.SubElement: Element(tag, attrib=...) sets tag and attrib, and SubElement appends a single named child observable via list(parent)"""
import xml.etree.ElementTree as ET

elem = ET.Element("person", attrib={"name": "Alice", "age": "30"})
assert elem.tag == "person", f"tag = {elem.tag!r}"
assert elem.attrib["name"] == "Alice", f"attrib = {elem.attrib!r}"

child = ET.SubElement(elem, "address")
child.text = "123 Main St"
assert len(list(elem)) == 1, "one child appended"
assert list(elem)[0].tag == "address", "child tag"
assert list(elem)[0].text == "123 Main St", "child text"

print("element_constructor_and_subelement OK")
"###);
    assert_output(&out, r###"element_constructor_and_subelement OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree/find_returns_first_findall_returns_all.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_find_returns_first_findall_returns_all() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "find_returns_first_findall_returns_all"
# subject = "xml.etree.ElementTree.Element.find"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.find: find returns the first matching child and findall returns every match in document order"""
import xml.etree.ElementTree as ET

zoo = ET.fromstring("<zoo><animal>lion</animal><animal>tiger</animal><animal>bear</animal></zoo>")
first = zoo.find("animal")
assert first is not None, "find returns an element"
assert first.text == "lion", f"first animal = {first.text!r}"
matches = zoo.findall("animal")
assert len(matches) == 3, f"findall count = {len(matches)!r}"
assert [e.text for e in matches] == ["lion", "tiger", "bear"], f"texts = {[e.text for e in matches]!r}"

print("find_returns_first_findall_returns_all OK")
"###);
    assert_output(&out, r###"find_returns_first_findall_returns_all OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree/fromstring_parses_attrib_text_children.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_fromstring_parses_attrib_text_children() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "fromstring_parses_attrib_text_children"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring parses the root tag, child attributes (as a dict), and per-child text exactly"""
import xml.etree.ElementTree as ET

xml = "<data><entry key='a' val='1'>text1</entry><entry key='b' val='2'>text2</entry></data>"
root = ET.fromstring(xml)
assert root.tag == "data", f"root tag = {root.tag!r}"
children = list(root)
assert len(children) == 2, f"child count = {len(children)!r}"
assert isinstance(children[0].attrib, dict), f"attrib type = {type(children[0].attrib)!r}"
assert children[0].attrib == {"key": "a", "val": "1"}, f"first attrib = {children[0].attrib!r}"
assert children[0].text == "text1", f"first text = {children[0].text!r}"
assert children[1].attrib == {"key": "b", "val": "2"}, f"second attrib = {children[1].attrib!r}"

print("fromstring_parses_attrib_text_children OK")
"###);
    assert_output(&out, r###"fromstring_parses_attrib_text_children OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree/fromstring_parses_many_children.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_fromstring_parses_many_children() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "fromstring_parses_many_children"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring on a root with 50 generated children preserves count and the first/last child id attributes"""
import xml.etree.ElementTree as ET

xml = "<root>" + "".join([f"<item id='{i}'>v{i}</item>" for i in range(50)]) + "</root>"
root = ET.fromstring(xml)
assert root.tag == "root", "root tag"
children = list(root)
assert len(children) == 50, f"50 children = {len(children)!r}"
assert children[0].attrib["id"] == "0", f"first id = {children[0].attrib['id']!r}"
assert children[0].text == "v0", f"first text = {children[0].text!r}"
assert children[49].attrib["id"] == "49", f"last id = {children[49].attrib['id']!r}"

print("fromstring_parses_many_children OK")
"###);
    assert_output(&out, r###"fromstring_parses_many_children OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree/iter_walks_descendants_including_self.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_iter_walks_descendants_including_self() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "iter_walks_descendants_including_self"
# subject = "xml.etree.ElementTree.Element.iter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.iter: iter() yields the element itself first, then every descendant tag in document order"""
import xml.etree.ElementTree as ET

root = ET.fromstring("<a><b><c/></b><d/></a>")
tags = [e.tag for e in root.iter()]
assert tags[0] == "a", "self first in iter"
assert tags == ["a", "b", "c", "d"], f"document-order tags = {tags!r}"
assert set(tags) == {"a", "b", "c", "d"}, f"all tags = {set(tags)!r}"

print("iter_walks_descendants_including_self OK")
"###);
    assert_output(&out, r###"iter_walks_descendants_including_self OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree/keys_lists_attrs_get_returns_value_or_default.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_keys_lists_attrs_get_returns_value_or_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "keys_lists_attrs_get_returns_value_or_default"
# subject = "xml.etree.ElementTree.Element.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.get: Element.keys() returns the attribute names and Element.get returns the value or the supplied default for a missing key"""
import xml.etree.ElementTree as ET

el = ET.fromstring("<el x='10' y='20' z='30'/>")
assert set(el.keys()) == {"x", "y", "z"}, f"keys = {set(el.keys())!r}"
assert el.get("x") == "10", f"get x = {el.get('x')!r}"
assert el.get("missing") is None, "get missing without default is None"
assert el.get("missing", "default") == "default", "get with default"

print("keys_lists_attrs_get_returns_value_or_default OK")
"###);
    assert_output(&out, r###"keys_lists_attrs_get_returns_value_or_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree/nested_element_text_and_tail.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_nested_element_text_and_tail() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "nested_element_text_and_tail"
# subject = "xml.etree.ElementTree.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element: for <outer>before<inner>inside</inner>after</outer>, outer.text is 'before', inner.text is 'inside', inner.tail is 'after'"""
import xml.etree.ElementTree as ET

outer = ET.fromstring("<outer>before<inner>inside</inner>after</outer>")
assert outer.text == "before", f"outer text = {outer.text!r}"
inner = list(outer)[0]
assert inner.text == "inside", f"inner text = {inner.text!r}"
assert inner.tail == "after", f"inner tail = {inner.tail!r}"

print("nested_element_text_and_tail OK")
"###);
    assert_output(&out, r###"nested_element_text_and_tail OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree/tostring_roundtrip_preserves_structure.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_tostring_roundtrip_preserves_structure() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "tostring_roundtrip_preserves_structure"
# subject = "xml.etree.ElementTree.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.tostring: tostring then fromstring round-trips the tag, child count, and attribute values; tostring returns bytes"""
import xml.etree.ElementTree as ET

root = ET.fromstring("<items><item id='1'/><item id='2'/></items>")
xml = ET.tostring(root)
assert isinstance(xml, bytes), f"tostring type = {type(xml)!r}"
assert b"<items>" in xml, "root tag in tostring output"

reparsed = ET.fromstring(xml)
assert reparsed.tag == "items", "round-trip tag"
assert len(list(reparsed)) == 2, "round-trip child count"
assert list(reparsed)[0].attrib["id"] == "1", "round-trip attrib"
assert list(reparsed)[1].attrib["id"] == "2", "round-trip attrib 2"

print("tostring_roundtrip_preserves_structure OK")
"###);
    assert_output(&out, r###"tostring_roundtrip_preserves_structure OK
"###);
}
