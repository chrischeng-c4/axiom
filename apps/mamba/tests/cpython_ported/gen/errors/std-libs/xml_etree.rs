use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/xml_etree/child_index_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_child_index_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "child_index_out_of_range_raises"
# subject = "xml.etree.ElementTree.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element: child_index_out_of_range_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<a><b/></a>")[99]
except IndexError:
    _raised = True
assert _raised, "child_index_out_of_range_raises: expected IndexError"
print("child_index_out_of_range_raises OK")
"###);
    assert_output(&out, r###"child_index_out_of_range_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/find_missing_tag_returns_none.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_find_missing_tag_returns_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "find_missing_tag_returns_none"
# subject = "xml.etree.ElementTree.Element.find"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.find: Element.find on an unknown tag returns None instead of raising"""
import xml.etree.ElementTree as ET

root = ET.fromstring("<a><b/></a>")
assert root.find("missing") is None, "find on unknown tag must return None"

print("find_missing_tag_returns_none OK")
"###);
    assert_output(&out, r###"find_missing_tag_returns_none OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/find_zero_predicate_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_find_zero_predicate_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "find_zero_predicate_raises"
# subject = "xml.etree.ElementTree.Element.find"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.find: find_zero_predicate_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<a><b/></a>").find("./b[0]")
except SyntaxError:
    _raised = True
assert _raised, "find_zero_predicate_raises: expected SyntaxError"
print("find_zero_predicate_raises OK")
"###);
    assert_output(&out, r###"find_zero_predicate_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/findall_absolute_path_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_findall_absolute_path_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "findall_absolute_path_raises"
# subject = "xml.etree.ElementTree.Element.findall"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.findall: findall_absolute_path_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<a><b/></a>").findall("/tag")
except SyntaxError:
    _raised = True
assert _raised, "findall_absolute_path_raises: expected SyntaxError"
print("findall_absolute_path_raises OK")
"###);
    assert_output(&out, r###"findall_absolute_path_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/findtext_missing_returns_default.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_findtext_missing_returns_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "findtext_missing_returns_default"
# subject = "xml.etree.ElementTree.Element.findtext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.findtext: Element.findtext on a missing path returns the supplied default value, not None"""
import xml.etree.ElementTree as ET

root = ET.fromstring("<a><b/></a>")
assert root.findtext("missing", "X") == "X", "findtext missing must return the default"
assert root.findtext("missing") is None, "findtext missing without default is None"

print("findtext_missing_returns_default OK")
"###);
    assert_output(&out, r###"findtext_missing_returns_default OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/fromstring_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_fromstring_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "fromstring_empty_raises"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring_empty_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("")
except ET.ParseError:
    _raised = True
assert _raised, "fromstring_empty_raises: expected ET.ParseError"
print("fromstring_empty_raises OK")
"###);
    assert_output(&out, r###"fromstring_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/fromstring_malformed_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_fromstring_malformed_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "fromstring_malformed_raises"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring_malformed_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<unclosed>text</wrong>")
except ET.ParseError:
    _raised = True
assert _raised, "fromstring_malformed_raises: expected ET.ParseError"
print("fromstring_malformed_raises OK")
"###);
    assert_output(&out, r###"fromstring_malformed_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/fromstring_undefined_entity_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_fromstring_undefined_entity_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "fromstring_undefined_entity_raises"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring_undefined_entity_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<document>&entity;</document>")
except ET.ParseError:
    _raised = True
assert _raised, "fromstring_undefined_entity_raises: expected ET.ParseError"
print("fromstring_undefined_entity_raises OK")
"###);
    assert_output(&out, r###"fromstring_undefined_entity_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/indent_negative_level_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_indent_negative_level_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "indent_negative_level_raises"
# subject = "xml.etree.ElementTree.indent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.indent: indent_negative_level_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.indent(ET.Element("x"), level=-1)
except ValueError:
    _raised = True
assert _raised, "indent_negative_level_raises: expected ValueError"
print("indent_negative_level_raises OK")
"###);
    assert_output(&out, r###"indent_negative_level_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/parse_missing_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_parse_missing_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "parse_missing_file_raises"
# subject = "xml.etree.ElementTree.parse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.parse: parse_missing_file_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.parse("/no/such/path/does-not-exist.xml")
except FileNotFoundError:
    _raised = True
assert _raised, "parse_missing_file_raises: expected FileNotFoundError"
print("parse_missing_file_raises OK")
"###);
    assert_output(&out, r###"parse_missing_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/parseerror_is_syntaxerror_subclass.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_parseerror_is_syntaxerror_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "parseerror_is_syntaxerror_subclass"
# subject = "xml.etree.ElementTree.ParseError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.ParseError: ET.ParseError is a subclass of the builtin SyntaxError"""
import xml.etree.ElementTree as ET

assert issubclass(ET.ParseError, SyntaxError), "ParseError must subclass SyntaxError"

print("parseerror_is_syntaxerror_subclass OK")
"###);
    assert_output(&out, r###"parseerror_is_syntaxerror_subclass OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/remove_absent_child_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_remove_absent_child_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "remove_absent_child_raises"
# subject = "xml.etree.ElementTree.Element.remove"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.remove: remove_absent_child_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.Element("a").remove(ET.Element("ghost"))
except ValueError:
    _raised = True
assert _raised, "remove_absent_child_raises: expected ValueError"
print("remove_absent_child_raises OK")
"###);
    assert_output(&out, r###"remove_absent_child_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/unqualified_child_default_namespace_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_unqualified_child_default_namespace_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "unqualified_child_default_namespace_raises"
# subject = "xml.etree.ElementTree.ElementTree.write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.ElementTree.write: serializing a default_namespace doc that contains an unqualified child raises ValueError"""
import io
import xml.etree.ElementTree as ET

bad = ET.Element("{default}elem")
ET.SubElement(bad, "plain")

_raised = False
try:
    buf = io.StringIO()
    ET.ElementTree(bad).write(buf, encoding="unicode", default_namespace="default")
except ValueError:
    _raised = True
assert _raised, "unqualified child + default_namespace must raise ValueError"

print("unqualified_child_default_namespace_raises OK")
"###);
    assert_output(&out, r###"unqualified_child_default_namespace_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/xml_etree/xpath_zero_position_raises.py`.
#[test]
fn test_gen_errors_std_libs_xml_etree_xpath_zero_position_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "xpath_zero_position_raises"
# subject = "xml.etree.ElementTree.Element.find"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.find: xpath_zero_position_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<body><tag/><tag/></body>").find("./tag[0]")
except SyntaxError:
    _raised = True
assert _raised, "xpath_zero_position_raises: expected SyntaxError"
print("xpath_zero_position_raises OK")
"###);
    assert_output(&out, r###"xpath_zero_position_raises OK
"###);
}
