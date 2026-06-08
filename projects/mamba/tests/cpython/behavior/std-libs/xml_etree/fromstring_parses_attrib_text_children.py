# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "fromstring_parses_attrib_text_children"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring parses the root tag, child attributes (as a dict), and per-child text exactly"""
import xml.etree.ElementTree as ET

xml = "<data><entry key='a' val='1'>text1</entry><entry key='b' val='2'>text2</entry></data>"
root = ET.fromstring(xml)
assert root.tag == "data", f"root tag = {root.tag!r}"
children = list(root)
assert len(children) == 2, f"child count = {len(children)!r}"
assert isinstance(children[0].attrib, dict), f"attrib type = {type(children[0].attrib)!r}"
assert children[0].attrib == {"key": "a", "val": "1"}, f"first attrib = {children[0].attrib!r}"
assert children[0].text == "text1", f"first text = {children[0].text!r}"
assert children[1].attrib == {"key": "b", "val": "2"}, f"second attrib = {children[1].attrib!r}"

print("fromstring_parses_attrib_text_children OK")
