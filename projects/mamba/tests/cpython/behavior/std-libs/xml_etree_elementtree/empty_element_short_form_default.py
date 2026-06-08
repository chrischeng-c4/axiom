# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "empty_element_short_form_default"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: an empty element collapses to '<tag />' by default and short_empty_elements=False forces explicit '<tag></tag>'"""
import xml.etree.ElementTree as ET

empty = ET.Element("empty")
assert ET.tostring(empty, encoding="unicode") == "<empty />", "default empty"
assert ET.tostring(empty, encoding="unicode", short_empty_elements=False) == "<empty></empty>", \
    "short_empty_elements off"

print("empty_element_short_form_default OK")
