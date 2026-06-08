# /// script
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
