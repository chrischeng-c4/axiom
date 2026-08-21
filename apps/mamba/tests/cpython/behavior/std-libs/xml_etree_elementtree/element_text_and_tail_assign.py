# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "element_text_and_tail_assign"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: assigning Element.text and Element.tail round-trips the exact strings back through the attributes"""
import xml.etree.ElementTree as ET

elem = ET.Element("item")
elem.text = "hello"
elem.tail = " world"
assert elem.text == "hello", f"text = {elem.text!r}"
assert elem.tail == " world", f"tail = {elem.tail!r}"

print("element_text_and_tail_assign OK")
