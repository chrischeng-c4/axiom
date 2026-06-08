# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "tostring_roundtrip_preserves_structure"
# subject = "xml.etree.ElementTree.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.tostring: tostring then fromstring round-trips the tag, child count, and attribute values; tostring returns bytes"""
import xml.etree.ElementTree as ET

root = ET.fromstring("<items><item id='1'/><item id='2'/></items>")
xml = ET.tostring(root)
assert isinstance(xml, bytes), f"tostring type = {type(xml)!r}"
assert b"<items>" in xml, "root tag in tostring output"

reparsed = ET.fromstring(xml)
assert reparsed.tag == "items", "round-trip tag"
assert len(list(reparsed)) == 2, "round-trip child count"
assert list(reparsed)[0].attrib["id"] == "1", "round-trip attrib"
assert list(reparsed)[1].attrib["id"] == "2", "round-trip attrib 2"

print("tostring_roundtrip_preserves_structure OK")
