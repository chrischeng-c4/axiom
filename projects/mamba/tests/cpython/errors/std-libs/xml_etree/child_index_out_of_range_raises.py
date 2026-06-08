# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "child_index_out_of_range_raises"
# subject = "xml.etree.ElementTree.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element: child_index_out_of_range_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<a><b/></a>")[99]
except IndexError:
    _raised = True
assert _raised, "child_index_out_of_range_raises: expected IndexError"
print("child_index_out_of_range_raises OK")
