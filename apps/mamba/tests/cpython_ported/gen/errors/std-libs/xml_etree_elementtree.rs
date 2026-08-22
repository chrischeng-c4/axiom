use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/xml_etree_elementtree/element_index_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_elementtree_element_index_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "element_index_out_of_range_raises"
# subject = "ET.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: element_index_out_of_range_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.Element("parent")[5]
except IndexError:
    _raised = True
assert _raised, "element_index_out_of_range_raises: expected IndexError"
print("element_index_out_of_range_raises OK")
"###);
    assert_output(&out, r###"element_index_out_of_range_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree_elementtree/element_none_index_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_elementtree_element_none_index_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "element_none_index_raises_typeerror"
# subject = "ET.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: element_none_index_raises_typeerror (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.Element("parent")[None]
except TypeError:
    _raised = True
assert _raised, "element_none_index_raises_typeerror: expected TypeError"
print("element_none_index_raises_typeerror OK")
"###);
    assert_output(&out, r###"element_none_index_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree_elementtree/fromstring_malformed_raises_parseerror.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_elementtree_fromstring_malformed_raises_parseerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "fromstring_malformed_raises_parseerror"
# subject = "ET.fromstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""ET.fromstring: fromstring_malformed_raises_parseerror (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<unclosed>")
except ET.ParseError:
    _raised = True
assert _raised, "fromstring_malformed_raises_parseerror: expected ET.ParseError"
print("fromstring_malformed_raises_parseerror OK")
"###);
    assert_output(&out, r###"fromstring_malformed_raises_parseerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree_elementtree/remove_non_child_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_elementtree_remove_non_child_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "remove_non_child_raises_valueerror"
# subject = "ET.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: remove_non_child_raises_valueerror (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.Element("parent").remove(ET.Element("other"))
except ValueError:
    _raised = True
assert _raised, "remove_non_child_raises_valueerror: expected ValueError"
print("remove_non_child_raises_valueerror OK")
"###);
    assert_output(&out, r###"remove_non_child_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree_elementtree/rootless_tree_write_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_elementtree_rootless_tree_write_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "rootless_tree_write_raises"
# subject = "ET.ElementTree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.ElementTree: an ElementTree() built with no root has getroot() is None, so write() raises (AttributeError under CPython 3.12) rather than emitting a document"""
import xml.etree.ElementTree as ET

import io
t = ET.ElementTree()
assert t.getroot() is None, "a rootless ElementTree has getroot() is None"
_raised = False
try:
    t.write(io.BytesIO())
except (AttributeError, TypeError):
    _raised = True
assert _raised, "rootless ElementTree.write() must raise"

print("rootless_tree_write_raises OK")
"###);
    assert_output(&out, r###"rootless_tree_write_raises OK
"###);
}
