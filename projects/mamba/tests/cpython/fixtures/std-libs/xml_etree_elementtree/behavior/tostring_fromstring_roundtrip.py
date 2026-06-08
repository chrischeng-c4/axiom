# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "tostring_fromstring_roundtrip"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: tostring(elem, encoding='unicode') serializes a tree that fromstring parses back to an equivalent tree (tag, child text, child attr preserved)"""
import xml.etree.ElementTree as ET

r = ET.Element("root")
c = ET.SubElement(r, "child", key="val")
c.text = "content"
s = ET.tostring(r, encoding="unicode")
rt = ET.fromstring(s)
assert rt.tag == "root", "round-trip tag"
assert rt.find("child").text == "content", "round-trip child text"
assert rt.find("child").get("key") == "val", "round-trip child attr"

print("tostring_fromstring_roundtrip OK")
