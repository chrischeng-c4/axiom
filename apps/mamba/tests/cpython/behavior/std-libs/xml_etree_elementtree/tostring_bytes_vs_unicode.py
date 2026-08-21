# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "tostring_bytes_vs_unicode"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: tostring(elem) returns bytes by default and tostring(elem, encoding='unicode') returns str"""
import xml.etree.ElementTree as ET

r = ET.Element("root")
s = ET.tostring(r)
assert isinstance(s, bytes), f"tostring type = {type(s)!r}"
assert b"<root" in s, f"tostring has tag: {s!r}"
su = ET.tostring(r, encoding="unicode")
assert isinstance(su, str), f"tostring unicode type = {type(su)!r}"

print("tostring_bytes_vs_unicode OK")
