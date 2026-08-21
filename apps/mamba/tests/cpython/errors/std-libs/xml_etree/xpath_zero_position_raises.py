# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "xpath_zero_position_raises"
# subject = "xml.etree.ElementTree.Element.find"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.find: xpath_zero_position_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<body><tag/><tag/></body>").find("./tag[0]")
except SyntaxError:
    _raised = True
assert _raised, "xpath_zero_position_raises: expected SyntaxError"
print("xpath_zero_position_raises OK")
