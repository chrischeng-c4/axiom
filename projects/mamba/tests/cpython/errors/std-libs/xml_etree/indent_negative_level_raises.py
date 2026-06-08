# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "indent_negative_level_raises"
# subject = "xml.etree.ElementTree.indent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.indent: indent_negative_level_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.indent(ET.Element("x"), level=-1)
except ValueError:
    _raised = True
assert _raised, "indent_negative_level_raises: expected ValueError"
print("indent_negative_level_raises OK")
