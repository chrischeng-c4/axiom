# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "xml_declaration_prepended"
# subject = "ET.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: tostring(elem, encoding='utf-8', xml_declaration=True) prepends the <?xml version='1.0' encoding='utf-8'?> header"""
import xml.etree.ElementTree as ET

decl = ET.tostring(ET.Element("doc"), encoding="utf-8", xml_declaration=True)
assert decl.startswith(b"<?xml version='1.0' encoding='utf-8'?>"), f"declaration = {decl!r}"

print("xml_declaration_prepended OK")
