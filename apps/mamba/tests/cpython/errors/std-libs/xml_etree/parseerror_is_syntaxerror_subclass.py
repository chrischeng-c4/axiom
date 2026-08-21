# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "parseerror_is_syntaxerror_subclass"
# subject = "xml.etree.ElementTree.ParseError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.ParseError: ET.ParseError is a subclass of the builtin SyntaxError"""
import xml.etree.ElementTree as ET

assert issubclass(ET.ParseError, SyntaxError), "ParseError must subclass SyntaxError"

print("parseerror_is_syntaxerror_subclass OK")
