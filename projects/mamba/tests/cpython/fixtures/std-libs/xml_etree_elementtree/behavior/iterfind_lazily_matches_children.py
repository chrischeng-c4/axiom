# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "iterfind_lazily_matches_children"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: iterfind(path) lazily yields only the children matching the XPath, skipping non-matching siblings"""
import xml.etree.ElementTree as ET

parent = ET.Element("parent")
for i in range(3):
    ET.SubElement(parent, "x", n=str(i))
ET.SubElement(parent, "y")
found = [e.get("n") for e in parent.iterfind("x")]
assert found == ["0", "1", "2"], f"iterfind = {found!r}"

print("iterfind_lazily_matches_children OK")
