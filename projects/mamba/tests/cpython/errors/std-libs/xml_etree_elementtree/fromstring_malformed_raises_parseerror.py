# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "fromstring_malformed_raises_parseerror"
# subject = "ET.fromstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""ET.fromstring: fromstring_malformed_raises_parseerror (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<unclosed>")
except ET.ParseError:
    _raised = True
assert _raised, "fromstring_malformed_raises_parseerror: expected ET.ParseError"
print("fromstring_malformed_raises_parseerror OK")
