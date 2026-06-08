# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "parse_missing_file_raises"
# subject = "xml.etree.ElementTree.parse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.parse: parse_missing_file_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.parse("/no/such/path/does-not-exist.xml")
except FileNotFoundError:
    _raised = True
assert _raised, "parse_missing_file_raises: expected FileNotFoundError"
print("parse_missing_file_raises OK")
