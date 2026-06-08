# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "cdata_delivered_as_plain_text"
# subject = "ET.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: CDATA section content is delivered as ordinary element text with no markup interpretation"""
import xml.etree.ElementTree as ET

cdata = ET.fromstring("<a><![CDATA[<raw> & stuff]]></a>")
assert cdata.text == "<raw> & stuff", f"cdata = {cdata.text!r}"

print("cdata_delivered_as_plain_text OK")
