# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "itertext_yields_text_in_document_order"
# subject = "ET.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: itertext() yields the text and tail strings of the element and its descendants in document order"""
import xml.etree.ElementTree as ET

r = ET.fromstring('<p>Hello <em>world</em> end</p>')
texts = list(r.itertext())
assert "Hello " in texts, f"p text = {texts!r}"
assert "world" in texts, f"em text = {texts!r}"
assert " end" in texts, f"em tail = {texts!r}"

print("itertext_yields_text_in_document_order OK")
