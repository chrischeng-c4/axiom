# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "findall_descendant_xpath"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""ET.Element: the './/c' descendant XPath matches every c anywhere under the root, and a [@id='..'] predicate selects by attribute"""
import xml.etree.ElementTree as ET

xml = '<a><b><c id="x">text</c></b><b><c id="y">more</c></b></a>'
r = ET.fromstring(xml)
cs = r.findall(".//c")  # all c descendants
assert len(cs) == 2, f"findall .//c = {len(cs)!r}"
assert cs[0].get("id") == "x", "first c"
assert r.findtext(".//c[@id='y']") == "more", "findtext by attr predicate"

print("findall_descendant_xpath OK")
