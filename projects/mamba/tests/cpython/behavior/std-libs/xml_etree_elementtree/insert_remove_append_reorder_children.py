# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "insert_remove_append_reorder_children"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: remove() drops a child and shifts the rest, insert(idx, elem) places a new child at the index, leaving the surrounding children in order"""
import xml.etree.ElementTree as ET

r = ET.Element("list")
for i in range(3):
    ET.SubElement(r, "item").text = str(i)
assert len(r) == 3, f"initial children = {len(r)!r}"
r.remove(r[1])  # remove middle
assert len(r) == 2, f"after remove = {len(r)!r}"
assert r[0].text == "0" and r[1].text == "2", "items after remove"
new = ET.Element("item")
new.text = "99"
r.insert(1, new)
assert r[1].text == "99", f"after insert: {r[1].text!r}"
assert [e.text for e in r] == ["0", "99", "2"], "order after insert"

print("insert_remove_append_reorder_children OK")
