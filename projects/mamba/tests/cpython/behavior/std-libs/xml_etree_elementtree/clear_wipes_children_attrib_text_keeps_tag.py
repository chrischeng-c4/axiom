# /// script
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
