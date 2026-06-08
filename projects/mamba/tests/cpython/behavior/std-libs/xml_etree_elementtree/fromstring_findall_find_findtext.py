# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "fromstring_findall_find_findtext"
# subject = "ET.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: fromstring parses a document; findall(tag) returns every direct child match, find(tag) returns the first, and findtext(tag) returns its text"""
import xml.etree.ElementTree as ET

xml = '<data><item id="1">Alice</item><item id="2">Bob</item></data>'
tree = ET.fromstring(xml)
assert tree.tag == "data", f"root tag = {tree.tag!r}"
items = tree.findall("item")
assert len(items) == 2, f"two items = {len(items)!r}"
assert items[0].get("id") == "1", "item[0] id"
assert items[0].text == "Alice", "item[0] text"
first = tree.find("item")
assert first is not None, "find returns element"
assert first.text == "Alice", "find first item"
assert tree.findtext("item") == "Alice", f"findtext = {tree.findtext('item')!r}"

print("fromstring_findall_find_findtext OK")
