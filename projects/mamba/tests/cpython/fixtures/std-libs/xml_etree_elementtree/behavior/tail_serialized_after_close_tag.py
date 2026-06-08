# /// script
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
