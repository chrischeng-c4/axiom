# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "fromstring_undefined_entity_raises"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: fromstring_undefined_entity_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<document>&entity;</document>")
except ET.ParseError:
    _raised = True
assert _raised, "fromstring_undefined_entity_raises: expected ET.ParseError"
print("fromstring_undefined_entity_raises OK")
