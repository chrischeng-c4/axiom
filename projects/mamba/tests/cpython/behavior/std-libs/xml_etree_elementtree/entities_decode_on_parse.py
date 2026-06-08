# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "entities_decode_on_parse"
# subject = "ET.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: named (&lt; &gt; &amp;) and numeric (&#65;) entities decode to their characters when an element is parsed"""
import xml.etree.ElementTree as ET

ent = ET.fromstring("<a>&lt;tag&gt; &amp; &#65;</a>")
assert ent.text == "<tag> & A", f"entities = {ent.text!r}"

print("entities_decode_on_parse OK")
