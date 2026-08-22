use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/attributes_serialize_in_insertion_order.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_attributes_serialize_in_insertion_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "attributes_serialize_in_insertion_order"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: attributes serialize in the order they were set (insertion order), not sorted"""
import xml.etree.ElementTree as ET

e = ET.Element("e")
e.set("z", "1")
e.set("a", "2")
e.set("m", "3")
out = ET.tostring(e, encoding="unicode")
assert out == '<e z="1" a="2" m="3" />', f"attr order = {out!r}"

print("attributes_serialize_in_insertion_order OK")
"###);
    assert_output(&out, r###"attributes_serialize_in_insertion_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/cdata_delivered_as_plain_text.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_cdata_delivered_as_plain_text() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "cdata_delivered_as_plain_text"
# subject = "ET.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: CDATA section content is delivered as ordinary element text with no markup interpretation"""
import xml.etree.ElementTree as ET

cdata = ET.fromstring("<a><![CDATA[<raw> & stuff]]></a>")
assert cdata.text == "<raw> & stuff", f"cdata = {cdata.text!r}"

print("cdata_delivered_as_plain_text OK")
"###);
    assert_output(&out, r###"cdata_delivered_as_plain_text OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/child_slice_read_returns_list.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_child_slice_read_returns_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "child_slice_read_returns_list"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: reading a slice of an Element returns a list of the matching child elements"""
import xml.etree.ElementTree as ET

root = ET.Element("root")
root.extend([ET.Element(t) for t in ("a", "b", "c", "d")])
middle = root[1:3]
assert [c.tag for c in middle] == ["b", "c"], f"slice = {[c.tag for c in middle]!r}"

print("child_slice_read_returns_list OK")
"###);
    assert_output(&out, r###"child_slice_read_returns_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/clear_wipes_children_attrib_text_keeps_tag.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_clear_wipes_children_attrib_text_keeps_tag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "clear_wipes_children_attrib_text_keeps_tag"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: Element.clear() removes all children, attributes, and text while keeping the tag"""
import xml.etree.ElementTree as ET

node = ET.Element("node", id="7")
node.text = "body"
ET.SubElement(node, "leaf")
node.clear()
assert len(node) == 0, "clear children"
assert node.attrib == {}, f"clear attrib = {node.attrib!r}"
assert node.text is None, f"clear text = {node.text!r}"
assert node.tag == "node", "clear keeps tag"

print("clear_wipes_children_attrib_text_keeps_tag OK")
"###);
    assert_output(&out, r###"clear_wipes_children_attrib_text_keeps_tag OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/comment_and_pi_tag_identity.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_comment_and_pi_tag_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "comment_and_pi_tag_identity"
# subject = "ET.Comment"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Comment: Comment(text) and ProcessingInstruction(target, text) produce nodes whose .tag is the factory itself and whose text is preserved"""
import xml.etree.ElementTree as ET

comment = ET.Comment("note")
assert comment.tag is ET.Comment, "comment tag identity"
assert comment.text == "note", f"comment text = {comment.text!r}"
pi = ET.ProcessingInstruction("target", "value")
assert pi.tag is ET.ProcessingInstruction, "pi tag identity"

print("comment_and_pi_tag_identity OK")
"###);
    assert_output(&out, r###"comment_and_pi_tag_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/comment_serializes_inside_marker.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_comment_serializes_inside_marker() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "comment_serializes_inside_marker"
# subject = "ET.Comment"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Comment: a Comment appended to an element serializes inside the <!-- --> marker"""
import xml.etree.ElementTree as ET

holder = ET.Element("doc")
holder.append(ET.Comment("hi"))
out = ET.tostring(holder, encoding="unicode")
assert out == "<doc><!--hi--></doc>", f"comment serialize = {out!r}"

print("comment_serializes_inside_marker OK")
"###);
    assert_output(&out, r###"comment_serializes_inside_marker OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/deepcopy_isolates_subtree.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_deepcopy_isolates_subtree() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "deepcopy_isolates_subtree"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: copy.deepcopy of an element yields an independent subtree; mutating the clone's text leaves the original unchanged"""
import xml.etree.ElementTree as ET

import copy
original = ET.Element("root")
ET.SubElement(original, "leaf").text = "orig"
clone = copy.deepcopy(original)
clone.find("leaf").text = "changed"
assert original.find("leaf").text == "orig", "deepcopy isolates original"
assert clone.find("leaf").text == "changed", "deepcopy mutates clone"

print("deepcopy_isolates_subtree OK")
"###);
    assert_output(&out, r###"deepcopy_isolates_subtree OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/del_child_by_index_and_slice.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_del_child_by_index_and_slice() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "del_child_by_index_and_slice"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: del elem[i] drops one child and shifts the rest; del elem[i:j] drops a contiguous range"""
import xml.etree.ElementTree as ET

root = ET.Element("root")
root.extend([ET.Element(t) for t in ("a", "b", "c", "d")])
del root[0]
assert [c.tag for c in root] == ["b", "c", "d"], "del index"
del root[0:2]
assert [c.tag for c in root] == ["d"], "del slice"

print("del_child_by_index_and_slice OK")
"###);
    assert_output(&out, r###"del_child_by_index_and_slice OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/element_get_returns_attr_or_default.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_element_get_returns_attr_or_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "element_get_returns_attr_or_default"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: Element(**attrs) records attributes; get(name) returns the value and get(missing, default) returns the supplied default"""
import xml.etree.ElementTree as ET

elem = ET.Element("item", id="1", name="Alice")
assert elem.attrib["id"] == "1", f"id attr = {elem.attrib['id']!r}"
assert elem.get("name") == "Alice", f"get name = {elem.get('name')!r}"
assert elem.get("missing", "default") == "default", "get default"
assert elem.get("missing") is None, "get missing without default is None"

print("element_get_returns_attr_or_default OK")
"###);
    assert_output(&out, r###"element_get_returns_attr_or_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/element_keys_and_values_reflect_attrib.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_element_keys_and_values_reflect_attrib() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "element_keys_and_values_reflect_attrib"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: Element.keys() and attrib.values() reflect the set of attribute names and values"""
import xml.etree.ElementTree as ET

elem = ET.Element("e", a="1", b="2")
assert set(elem.keys()) == {"a", "b"}, f"keys = {set(elem.keys())!r}"
assert set(elem.attrib.values()) == {"1", "2"}, f"values = {set(elem.attrib.values())!r}"

print("element_keys_and_values_reflect_attrib OK")
"###);
    assert_output(&out, r###"element_keys_and_values_reflect_attrib OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/element_set_and_get_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_element_set_and_get_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "element_set_and_get_roundtrip"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: Element.set(name, value) makes the attribute observable via get(name)"""
import xml.etree.ElementTree as ET

tree = ET.fromstring('<data><item id="1">Alice</item></data>')
tree.set("version", "1.0")
assert tree.get("version") == "1.0", f"set attr = {tree.get('version')!r}"

print("element_set_and_get_roundtrip OK")
"###);
    assert_output(&out, r###"element_set_and_get_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/element_tag_text_attrib_defaults.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_element_tag_text_attrib_defaults() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "element_tag_text_attrib_defaults"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: a freshly created Element exposes its tag, has text None and an empty attrib dict by default"""
import xml.etree.ElementTree as ET

root = ET.Element("root")
assert isinstance(root, ET.Element), f"Element type = {type(root)!r}"
assert root.tag == "root", f"tag = {root.tag!r}"
assert root.text is None, f"initial text = {root.text!r}"
assert root.attrib == {}, f"initial attrib = {root.attrib!r}"

print("element_tag_text_attrib_defaults OK")
"###);
    assert_output(&out, r###"element_tag_text_attrib_defaults OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/element_text_and_tail_assign.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_element_text_and_tail_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "element_text_and_tail_assign"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: assigning Element.text and Element.tail round-trips the exact strings back through the attributes"""
import xml.etree.ElementTree as ET

elem = ET.Element("item")
elem.text = "hello"
elem.tail = " world"
assert elem.text == "hello", f"text = {elem.text!r}"
assert elem.tail == " world", f"tail = {elem.tail!r}"

print("element_text_and_tail_assign OK")
"###);
    assert_output(&out, r###"element_text_and_tail_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/elementtree_write_serializes_to_filelike.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_elementtree_write_serializes_to_filelike() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "elementtree_write_serializes_to_filelike"
# subject = "ET.ElementTree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.ElementTree: ElementTree(root).write(buf) serializes the document into a binary file-like, emitting the open tags and text content"""
import xml.etree.ElementTree as ET

import io
buf = io.BytesIO()
r = ET.Element("doc")
ET.SubElement(r, "note").text = "hello"
et = ET.ElementTree(r)
et.write(buf)
buf.seek(0)
out = buf.read()
assert b"<doc>" in out, f"write output: {out!r}"
assert b"<note>" in out and b"hello" in out, "note in output"

print("elementtree_write_serializes_to_filelike OK")
"###);
    assert_output(&out, r###"elementtree_write_serializes_to_filelike OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/empty_element_short_form_default.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_empty_element_short_form_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "empty_element_short_form_default"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: an empty element collapses to '<tag />' by default and short_empty_elements=False forces explicit '<tag></tag>'"""
import xml.etree.ElementTree as ET

empty = ET.Element("empty")
assert ET.tostring(empty, encoding="unicode") == "<empty />", "default empty"
assert ET.tostring(empty, encoding="unicode", short_empty_elements=False) == "<empty></empty>", \
    "short_empty_elements off"

print("empty_element_short_form_default OK")
"###);
    assert_output(&out, r###"empty_element_short_form_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/entities_decode_on_parse.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_entities_decode_on_parse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "entities_decode_on_parse"
# subject = "ET.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: named (&lt; &gt; &amp;) and numeric (&#65;) entities decode to their characters when an element is parsed"""
import xml.etree.ElementTree as ET

ent = ET.fromstring("<a>&lt;tag&gt; &amp; &#65;</a>")
assert ent.text == "<tag> & A", f"entities = {ent.text!r}"

print("entities_decode_on_parse OK")
"###);
    assert_output(&out, r###"entities_decode_on_parse OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/extend_appends_children_in_order.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_extend_appends_children_in_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "extend_appends_children_in_order"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: extend(iterable) appends the children in order and append() adds a single child after them"""
import xml.etree.ElementTree as ET

root = ET.Element("root")
root.extend([ET.Element("a"), ET.Element("b"), ET.Element("c")])
assert len(root) == 3, f"extend len = {len(root)!r}"
assert [c.tag for c in root] == ["a", "b", "c"], "extend order"
root.append(ET.Element("d"))
assert [c.tag for c in root] == ["a", "b", "c", "d"], "append tail"

print("extend_appends_children_in_order OK")
"###);
    assert_output(&out, r###"extend_appends_children_in_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/findall_descendant_xpath.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_findall_descendant_xpath() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "findall_descendant_xpath"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""ET.Element: the './/c' descendant XPath matches every c anywhere under the root, and a [@id='..'] predicate selects by attribute"""
import xml.etree.ElementTree as ET

xml = '<a><b><c id="x">text</c></b><b><c id="y">more</c></b></a>'
r = ET.fromstring(xml)
cs = r.findall(".//c")  # all c descendants
assert len(cs) == 2, f"findall .//c = {len(cs)!r}"
assert cs[0].get("id") == "x", "first c"
assert r.findtext(".//c[@id='y']") == "more", "findtext by attr predicate"

print("findall_descendant_xpath OK")
"###);
    assert_output(&out, r###"findall_descendant_xpath OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/fromstring_findall_find_findtext.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_fromstring_findall_find_findtext() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "fromstring_findall_find_findtext"
# subject = "ET.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: fromstring parses a document; findall(tag) returns every direct child match, find(tag) returns the first, and findtext(tag) returns its text"""
import xml.etree.ElementTree as ET

xml = '<data><item id="1">Alice</item><item id="2">Bob</item></data>'
tree = ET.fromstring(xml)
assert tree.tag == "data", f"root tag = {tree.tag!r}"
items = tree.findall("item")
assert len(items) == 2, f"two items = {len(items)!r}"
assert items[0].get("id") == "1", "item[0] id"
assert items[0].text == "Alice", "item[0] text"
first = tree.find("item")
assert first is not None, "find returns element"
assert first.text == "Alice", "find first item"
assert tree.findtext("item") == "Alice", f"findtext = {tree.findtext('item')!r}"

print("fromstring_findall_find_findtext OK")
"###);
    assert_output(&out, r###"fromstring_findall_find_findtext OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/indent_pretty_prints_two_space_nesting.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_indent_pretty_prints_two_space_nesting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "indent_pretty_prints_two_space_nesting"
# subject = "ET.indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.indent: indent(tree) rewrites text/tail so tostring pretty-prints with newlines and two-space-per-level nesting"""
import xml.etree.ElementTree as ET

tree = ET.Element("root")
inner = ET.SubElement(tree, "a")
ET.SubElement(inner, "b")
ET.indent(tree)
out = ET.tostring(tree, encoding="unicode")
assert out == "<root>\n  <a>\n    <b />\n  </a>\n</root>", f"indent = {out!r}"

print("indent_pretty_prints_two_space_nesting OK")
"###);
    assert_output(&out, r###"indent_pretty_prints_two_space_nesting OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/insert_remove_append_reorder_children.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_insert_remove_append_reorder_children() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "insert_remove_append_reorder_children"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: remove() drops a child and shifts the rest, insert(idx, elem) places a new child at the index, leaving the surrounding children in order"""
import xml.etree.ElementTree as ET

r = ET.Element("list")
for i in range(3):
    ET.SubElement(r, "item").text = str(i)
assert len(r) == 3, f"initial children = {len(r)!r}"
r.remove(r[1])  # remove middle
assert len(r) == 2, f"after remove = {len(r)!r}"
assert r[0].text == "0" and r[1].text == "2", "items after remove"
new = ET.Element("item")
new.text = "99"
r.insert(1, new)
assert r[1].text == "99", f"after insert: {r[1].text!r}"
assert [e.text for e in r] == ["0", "99", "2"], "order after insert"

print("insert_remove_append_reorder_children OK")
"###);
    assert_output(&out, r###"insert_remove_append_reorder_children OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/iter_yields_all_elements.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_iter_yields_all_elements() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "iter_yields_all_elements"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: Element.iter() walks the whole subtree in document order, counting the root plus every descendant"""
import xml.etree.ElementTree as ET

tree = ET.fromstring('<data><item id="1">Alice</item><item id="2">Bob</item></data>')
all_elems = list(tree.iter())
# data + two items
assert len(all_elems) == 3, f"iter count = {len(all_elems)!r}"
assert [e.tag for e in all_elems] == ["data", "item", "item"], "iter tags in document order"

print("iter_yields_all_elements OK")
"###);
    assert_output(&out, r###"iter_yields_all_elements OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/iterfind_lazily_matches_children.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_iterfind_lazily_matches_children() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "iterfind_lazily_matches_children"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: iterfind(path) lazily yields only the children matching the XPath, skipping non-matching siblings"""
import xml.etree.ElementTree as ET

parent = ET.Element("parent")
for i in range(3):
    ET.SubElement(parent, "x", n=str(i))
ET.SubElement(parent, "y")
found = [e.get("n") for e in parent.iterfind("x")]
assert found == ["0", "1", "2"], f"iterfind = {found!r}"

print("iterfind_lazily_matches_children OK")
"###);
    assert_output(&out, r###"iterfind_lazily_matches_children OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/itertext_yields_text_in_document_order.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_itertext_yields_text_in_document_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "itertext_yields_text_in_document_order"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: itertext() yields the text and tail strings of the element and its descendants in document order"""
import xml.etree.ElementTree as ET

r = ET.fromstring('<p>Hello <em>world</em> end</p>')
texts = list(r.itertext())
assert "Hello " in texts, f"p text = {texts!r}"
assert "world" in texts, f"em text = {texts!r}"
assert " end" in texts, f"em tail = {texts!r}"

print("itertext_yields_text_in_document_order OK")
"###);
    assert_output(&out, r###"itertext_yields_text_in_document_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/parse_filelike_builds_tree.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_parse_filelike_builds_tree() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "parse_filelike_builds_tree"
# subject = "ET.parse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""ET.parse: parse() reads XML from a file-like object and the resulting ElementTree's getroot() reflects the document's tag, attributes, and nested structure"""
import xml.etree.ElementTree as ET

import io
xml = b"""<?xml version="1.0"?>
<catalog>
  <book id="1">
    <title>Python</title>
    <price>29.99</price>
  </book>
  <book id="2">
    <title>Rust</title>
    <price>39.99</price>
  </book>
</catalog>
"""
tree = ET.parse(io.BytesIO(xml))
root = tree.getroot()
assert root.tag == "catalog", f"root tag = {root.tag!r}"
books = root.findall("book")
assert len(books) == 2, f"two books = {len(books)!r}"
assert books[0].get("id") == "1", "book[0] id"
assert books[0].find("title").text == "Python", "first book title"

print("parse_filelike_builds_tree OK")
"###);
    assert_output(&out, r###"parse_filelike_builds_tree OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/qname_clark_notation.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_qname_clark_notation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "qname_clark_notation"
# subject = "ET.QName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.QName: QName(uri, local) joins a namespace URI and local name into Clark notation '{uri}local'"""
import xml.etree.ElementTree as ET

q = ET.QName("http://example.com", "tag")
assert q.text == "{http://example.com}tag", f"qname = {q.text!r}"

print("qname_clark_notation OK")
"###);
    assert_output(&out, r###"qname_clark_notation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/register_namespace_prefix_in_output.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_register_namespace_prefix_in_output() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "register_namespace_prefix_in_output"
# subject = "ET.register_namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.register_namespace: register_namespace maps a URI to a prefix so a Clark-notation '{uri}tag' element serializes with that prefix"""
import xml.etree.ElementTree as ET

ET.register_namespace("ns", "http://example.com/ns")
r = ET.Element("{http://example.com/ns}root")
ET.SubElement(r, "{http://example.com/ns}child").text = "data"
s = ET.tostring(r, encoding="unicode")
assert "ns:root" in s, f"registered prefix in output: {s!r}"
assert "ns:child" in s, f"registered prefix on child: {s!r}"

print("register_namespace_prefix_in_output OK")
"###);
    assert_output(&out, r###"register_namespace_prefix_in_output OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/subelement_appends_and_counts_child.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_subelement_appends_and_counts_child() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "subelement_appends_and_counts_child"
# subject = "ET.SubElement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.SubElement: SubElement(parent, tag, **attrs) creates a child carrying the tag/attrs and appends it so len(parent) grows by one"""
import xml.etree.ElementTree as ET

root = ET.Element("root")
child = ET.SubElement(root, "child", value="42")
assert child.tag == "child", f"child tag = {child.tag!r}"
assert child.get("value") == "42", f"child attr = {child.get('value')!r}"
assert len(root) == 1, f"root has one child = {len(root)!r}"
assert root[0] is child, "the child is the first element of the parent"

print("subelement_appends_and_counts_child OK")
"###);
    assert_output(&out, r###"subelement_appends_and_counts_child OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/tail_serialized_after_close_tag.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_tail_serialized_after_close_tag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "tail_serialized_after_close_tag"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: a child's tail text serializes immediately after its closing tag"""
import xml.etree.ElementTree as ET

root = ET.Element("r")
child = ET.SubElement(root, "c")
child.text = "x"
child.tail = "TAIL"
out = ET.tostring(root, encoding="unicode")
assert out == "<r><c>x</c>TAIL</r>", f"tail = {out!r}"

print("tail_serialized_after_close_tag OK")
"###);
    assert_output(&out, r###"tail_serialized_after_close_tag OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/tostring_bytes_vs_unicode.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_tostring_bytes_vs_unicode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "tostring_bytes_vs_unicode"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: tostring(elem) returns bytes by default and tostring(elem, encoding='unicode') returns str"""
import xml.etree.ElementTree as ET

r = ET.Element("root")
s = ET.tostring(r)
assert isinstance(s, bytes), f"tostring type = {type(s)!r}"
assert b"<root" in s, f"tostring has tag: {s!r}"
su = ET.tostring(r, encoding="unicode")
assert isinstance(su, str), f"tostring unicode type = {type(su)!r}"

print("tostring_bytes_vs_unicode OK")
"###);
    assert_output(&out, r###"tostring_bytes_vs_unicode OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/tostring_fromstring_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_tostring_fromstring_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "tostring_fromstring_roundtrip"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: tostring(elem, encoding='unicode') serializes a tree that fromstring parses back to an equivalent tree (tag, child text, child attr preserved)"""
import xml.etree.ElementTree as ET

r = ET.Element("root")
c = ET.SubElement(r, "child", key="val")
c.text = "content"
s = ET.tostring(r, encoding="unicode")
rt = ET.fromstring(s)
assert rt.tag == "root", "round-trip tag"
assert rt.find("child").text == "content", "round-trip child text"
assert rt.find("child").get("key") == "val", "round-trip child attr"

print("tostring_fromstring_roundtrip OK")
"###);
    assert_output(&out, r###"tostring_fromstring_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/treebuilder_builds_tree_imperatively.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_treebuilder_builds_tree_imperatively() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "treebuilder_builds_tree_imperatively"
# subject = "ET.TreeBuilder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.TreeBuilder: TreeBuilder.start/data/end/close builds an element tree imperatively, carrying tag, attributes, and text"""
import xml.etree.ElementTree as ET

builder = ET.TreeBuilder()
builder.start("a", {"k": "v"})
builder.data("text")
builder.end("a")
built = builder.close()
assert built.tag == "a", "builder tag"
assert built.get("k") == "v", "builder attr"
assert built.text == "text", "builder data"

print("treebuilder_builds_tree_imperatively OK")
"###);
    assert_output(&out, r###"treebuilder_builds_tree_imperatively OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/xml_declaration_prepended.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_xml_declaration_prepended() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "xml_declaration_prepended"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: tostring(elem, encoding='utf-8', xml_declaration=True) prepends the <?xml version='1.0' encoding='utf-8'?> header"""
import xml.etree.ElementTree as ET

decl = ET.tostring(ET.Element("doc"), encoding="utf-8", xml_declaration=True)
assert decl.startswith(b"<?xml version='1.0' encoding='utf-8'?>"), f"declaration = {decl!r}"

print("xml_declaration_prepended OK")
"###);
    assert_output(&out, r###"xml_declaration_prepended OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_etree_elementtree/xmlparser_feed_chunks_close.py`.
#[test]
fn test_gen_behavior_std_libs_xml_etree_elementtree_xmlparser_feed_chunks_close() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "xmlparser_feed_chunks_close"
# subject = "ET.XMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.XMLParser: XMLParser.feed() accepts a document split into arbitrary chunks and close() returns the assembled root with all children"""
import xml.etree.ElementTree as ET

parser = ET.XMLParser()
parser.feed("<root>")
parser.feed("<item>one</item>")
parser.feed("<item>two</item>")
parser.feed("</root>")
root = parser.close()
assert root.tag == "root", f"feed root = {root.tag!r}"
assert [c.text for c in root] == ["one", "two"], "feed children"

print("xmlparser_feed_chunks_close OK")
"###);
    assert_output(&out, r###"xmlparser_feed_chunks_close OK
"###);
}
