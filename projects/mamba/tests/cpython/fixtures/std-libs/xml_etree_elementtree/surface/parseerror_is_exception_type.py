# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "parseerror_is_exception_type"
# subject = "ET.ParseError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.ParseError: parseerror_is_exception_type (surface)."""
import xml.etree.ElementTree as ET

assert hasattr(ET.ParseError, "__cause__")
print("parseerror_is_exception_type OK")
