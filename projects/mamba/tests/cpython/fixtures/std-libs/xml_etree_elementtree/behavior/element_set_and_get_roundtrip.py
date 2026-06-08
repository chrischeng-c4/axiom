# /// script
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
