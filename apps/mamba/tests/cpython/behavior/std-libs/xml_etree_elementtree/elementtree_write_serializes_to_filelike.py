# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "elementtree_write_serializes_to_filelike"
# subject = "ET.ElementTree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.ElementTree: ElementTree(root).write(buf) serializes the document into a binary file-like, emitting the open tags and text content"""
import xml.etree.ElementTree as ET

import io
buf = io.BytesIO()
r = ET.Element("doc")
ET.SubElement(r, "note").text = "hello"
et = ET.ElementTree(r)
et.write(buf)
buf.seek(0)
out = buf.read()
assert b"<doc>" in out, f"write output: {out!r}"
assert b"<note>" in out and b"hello" in out, "note in output"

print("elementtree_write_serializes_to_filelike OK")
