# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "xmlparser_feed_chunks_close"
# subject = "ET.XMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.XMLParser: XMLParser.feed() accepts a document split into arbitrary chunks and close() returns the assembled root with all children"""
import xml.etree.ElementTree as ET

parser = ET.XMLParser()
parser.feed("<root>")
parser.feed("<item>one</item>")
parser.feed("<item>two</item>")
parser.feed("</root>")
root = parser.close()
assert root.tag == "root", f"feed root = {root.tag!r}"
assert [c.text for c in root] == ["one", "two"], "feed children"

print("xmlparser_feed_chunks_close OK")
