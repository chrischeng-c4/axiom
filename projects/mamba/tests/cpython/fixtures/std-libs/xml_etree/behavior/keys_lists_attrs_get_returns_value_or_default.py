# /// script
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
