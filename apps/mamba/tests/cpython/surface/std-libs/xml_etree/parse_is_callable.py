# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "surface"
# case = "parse_is_callable"
# subject = "xml.etree.ElementTree.parse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""xml.etree.ElementTree.parse: parse_is_callable (surface)."""
import xml.etree.ElementTree

assert callable(xml.etree.ElementTree.parse)
print("parse_is_callable OK")
