# /// script
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
