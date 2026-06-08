# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "find_zero_predicate_raises"
# subject = "xml.etree.ElementTree.Element.find"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.find: find_zero_predicate_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<a><b/></a>").find("./b[0]")
except SyntaxError:
    _raised = True
assert _raised, "find_zero_predicate_raises: expected SyntaxError"
print("find_zero_predicate_raises OK")
