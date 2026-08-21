# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "surface"
# case = "parseerror_is_exception_type"
# subject = "xml.etree.ElementTree.ParseError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""xml.etree.ElementTree.ParseError: parseerror_is_exception_type (surface)."""
import xml.etree.ElementTree

assert type(xml.etree.ElementTree.ParseError).__name__ == "type"
print("parseerror_is_exception_type OK")
