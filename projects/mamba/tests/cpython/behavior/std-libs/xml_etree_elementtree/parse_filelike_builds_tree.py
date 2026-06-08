# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "parse_filelike_builds_tree"
# subject = "ET.parse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""ET.parse: parse() reads XML from a file-like object and the resulting ElementTree's getroot() reflects the document's tag, attributes, and nested structure"""
import xml.etree.ElementTree as ET

import io
xml = b"""<?xml version="1.0"?>
<catalog>
  <book id="1">
    <title>Python</title>
    <price>29.99</price>
  </book>
  <book id="2">
    <title>Rust</title>
    <price>39.99</price>
  </book>
</catalog>
"""
tree = ET.parse(io.BytesIO(xml))
root = tree.getroot()
assert root.tag == "catalog", f"root tag = {root.tag!r}"
books = root.findall("book")
assert len(books) == 2, f"two books = {len(books)!r}"
assert books[0].get("id") == "1", "book[0] id"
assert books[0].find("title").text == "Python", "first book title"

print("parse_filelike_builds_tree OK")
