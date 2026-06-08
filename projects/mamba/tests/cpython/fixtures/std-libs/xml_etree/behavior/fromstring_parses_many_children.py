# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "fromstring_parses_many_children"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring on a root with 50 generated children preserves count and the first/last child id attributes"""
import xml.etree.ElementTree as ET

xml = "<root>" + "".join([f"<item id='{i}'>v{i}</item>" for i in range(50)]) + "</root>"
root = ET.fromstring(xml)
assert root.tag == "root", "root tag"
children = list(root)
assert len(children) == 50, f"50 children = {len(children)!r}"
assert children[0].attrib["id"] == "0", f"first id = {children[0].attrib['id']!r}"
assert children[0].text == "v0", f"first text = {children[0].text!r}"
assert children[49].attrib["id"] == "49", f"last id = {children[49].attrib['id']!r}"

print("fromstring_parses_many_children OK")
