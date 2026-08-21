# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "iter_yields_all_elements"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: Element.iter() walks the whole subtree in document order, counting the root plus every descendant"""
import xml.etree.ElementTree as ET

tree = ET.fromstring('<data><item id="1">Alice</item><item id="2">Bob</item></data>')
all_elems = list(tree.iter())
# data + two items
assert len(all_elems) == 3, f"iter count = {len(all_elems)!r}"
assert [e.tag for e in all_elems] == ["data", "item", "item"], "iter tags in document order"

print("iter_yields_all_elements OK")
