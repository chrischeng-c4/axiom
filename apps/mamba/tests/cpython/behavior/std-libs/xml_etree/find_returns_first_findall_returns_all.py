# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "find_returns_first_findall_returns_all"
# subject = "xml.etree.ElementTree.Element.find"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.find: find returns the first matching child and findall returns every match in document order"""
import xml.etree.ElementTree as ET

zoo = ET.fromstring("<zoo><animal>lion</animal><animal>tiger</animal><animal>bear</animal></zoo>")
first = zoo.find("animal")
assert first is not None, "find returns an element"
assert first.text == "lion", f"first animal = {first.text!r}"
matches = zoo.findall("animal")
assert len(matches) == 3, f"findall count = {len(matches)!r}"
assert [e.text for e in matches] == ["lion", "tiger", "bear"], f"texts = {[e.text for e in matches]!r}"

print("find_returns_first_findall_returns_all OK")
