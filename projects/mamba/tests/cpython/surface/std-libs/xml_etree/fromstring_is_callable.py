# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "surface"
# case = "fromstring_is_callable"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring_is_callable (surface)."""
import xml.etree.ElementTree

assert callable(xml.etree.ElementTree.fromstring)
print("fromstring_is_callable OK")
