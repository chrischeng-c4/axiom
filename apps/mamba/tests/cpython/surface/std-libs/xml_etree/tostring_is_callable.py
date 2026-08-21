# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "surface"
# case = "tostring_is_callable"
# subject = "xml.etree.ElementTree.tostring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""xml.etree.ElementTree.tostring: tostring_is_callable (surface)."""
import xml.etree.ElementTree

assert callable(xml.etree.ElementTree.tostring)
print("tostring_is_callable OK")
