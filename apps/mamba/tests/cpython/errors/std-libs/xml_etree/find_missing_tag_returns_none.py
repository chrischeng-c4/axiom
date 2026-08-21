# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "find_missing_tag_returns_none"
# subject = "xml.etree.ElementTree.Element.find"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.find: Element.find on an unknown tag returns None instead of raising"""
import xml.etree.ElementTree as ET

root = ET.fromstring("<a><b/></a>")
assert root.find("missing") is None, "find on unknown tag must return None"

print("find_missing_tag_returns_none OK")
