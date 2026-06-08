# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "comment_serializes_inside_marker"
# subject = "ET.Comment"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Comment: a Comment appended to an element serializes inside the <!-- --> marker"""
import xml.etree.ElementTree as ET

holder = ET.Element("doc")
holder.append(ET.Comment("hi"))
out = ET.tostring(holder, encoding="unicode")
assert out == "<doc><!--hi--></doc>", f"comment serialize = {out!r}"

print("comment_serializes_inside_marker OK")
