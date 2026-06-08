# /// script
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
