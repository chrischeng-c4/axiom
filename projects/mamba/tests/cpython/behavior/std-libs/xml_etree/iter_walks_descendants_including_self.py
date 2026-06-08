# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "iter_walks_descendants_including_self"
# subject = "xml.etree.ElementTree.Element.iter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.iter: iter() yields the element itself first, then every descendant tag in document order"""
import xml.etree.ElementTree as ET

root = ET.fromstring("<a><b><c/></b><d/></a>")
tags = [e.tag for e in root.iter()]
assert tags[0] == "a", "self first in iter"
assert tags == ["a", "b", "c", "d"], f"document-order tags = {tags!r}"
assert set(tags) == {"a", "b", "c", "d"}, f"all tags = {set(tags)!r}"

print("iter_walks_descendants_including_self OK")
