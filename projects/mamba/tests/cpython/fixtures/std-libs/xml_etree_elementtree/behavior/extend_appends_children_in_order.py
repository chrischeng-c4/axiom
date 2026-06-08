# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "extend_appends_children_in_order"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: extend(iterable) appends the children in order and append() adds a single child after them"""
import xml.etree.ElementTree as ET

root = ET.Element("root")
root.extend([ET.Element("a"), ET.Element("b"), ET.Element("c")])
assert len(root) == 3, f"extend len = {len(root)!r}"
assert [c.tag for c in root] == ["a", "b", "c"], "extend order"
root.append(ET.Element("d"))
assert [c.tag for c in root] == ["a", "b", "c", "d"], "append tail"

print("extend_appends_children_in_order OK")
