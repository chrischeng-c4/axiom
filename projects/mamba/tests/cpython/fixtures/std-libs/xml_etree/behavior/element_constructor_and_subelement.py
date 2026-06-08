# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "element_constructor_and_subelement"
# subject = "xml.etree.ElementTree.SubElement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.SubElement: Element(tag, attrib=...) sets tag and attrib, and SubElement appends a single named child observable via list(parent)"""
import xml.etree.ElementTree as ET

elem = ET.Element("person", attrib={"name": "Alice", "age": "30"})
assert elem.tag == "person", f"tag = {elem.tag!r}"
assert elem.attrib["name"] == "Alice", f"attrib = {elem.attrib!r}"

child = ET.SubElement(elem, "address")
child.text = "123 Main St"
assert len(list(elem)) == 1, "one child appended"
assert list(elem)[0].tag == "address", "child tag"
assert list(elem)[0].text == "123 Main St", "child text"

print("element_constructor_and_subelement OK")
