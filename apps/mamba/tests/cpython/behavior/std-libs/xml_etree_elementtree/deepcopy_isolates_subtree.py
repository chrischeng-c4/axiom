# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "deepcopy_isolates_subtree"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: copy.deepcopy of an element yields an independent subtree; mutating the clone's text leaves the original unchanged"""
import xml.etree.ElementTree as ET

import copy
original = ET.Element("root")
ET.SubElement(original, "leaf").text = "orig"
clone = copy.deepcopy(original)
clone.find("leaf").text = "changed"
assert original.find("leaf").text == "orig", "deepcopy isolates original"
assert clone.find("leaf").text == "changed", "deepcopy mutates clone"

print("deepcopy_isolates_subtree OK")
