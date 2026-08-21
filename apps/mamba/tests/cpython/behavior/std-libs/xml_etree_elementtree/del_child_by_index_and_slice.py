# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "del_child_by_index_and_slice"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: del elem[i] drops one child and shifts the rest; del elem[i:j] drops a contiguous range"""
import xml.etree.ElementTree as ET

root = ET.Element("root")
root.extend([ET.Element(t) for t in ("a", "b", "c", "d")])
del root[0]
assert [c.tag for c in root] == ["b", "c", "d"], "del index"
del root[0:2]
assert [c.tag for c in root] == ["d"], "del slice"

print("del_child_by_index_and_slice OK")
