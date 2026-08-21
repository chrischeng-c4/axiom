# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "element_tag_text_attrib_defaults"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: a freshly created Element exposes its tag, has text None and an empty attrib dict by default"""
import xml.etree.ElementTree as ET

root = ET.Element("root")
assert isinstance(root, ET.Element), f"Element type = {type(root)!r}"
assert root.tag == "root", f"tag = {root.tag!r}"
assert root.text is None, f"initial text = {root.text!r}"
assert root.attrib == {}, f"initial attrib = {root.attrib!r}"

print("element_tag_text_attrib_defaults OK")
