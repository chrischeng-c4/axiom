# /// script
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
