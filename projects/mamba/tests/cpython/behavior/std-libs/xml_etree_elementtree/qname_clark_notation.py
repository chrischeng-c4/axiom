# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "qname_clark_notation"
# subject = "ET.QName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.QName: QName(uri, local) joins a namespace URI and local name into Clark notation '{uri}local'"""
import xml.etree.ElementTree as ET

q = ET.QName("http://example.com", "tag")
assert q.text == "{http://example.com}tag", f"qname = {q.text!r}"

print("qname_clark_notation OK")
