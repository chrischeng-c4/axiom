# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "attributes_serialize_in_insertion_order"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: attributes serialize in the order they were set (insertion order), not sorted"""
import xml.etree.ElementTree as ET

e = ET.Element("e")
e.set("z", "1")
e.set("a", "2")
e.set("m", "3")
out = ET.tostring(e, encoding="unicode")
assert out == '<e z="1" a="2" m="3" />', f"attr order = {out!r}"

print("attributes_serialize_in_insertion_order OK")
