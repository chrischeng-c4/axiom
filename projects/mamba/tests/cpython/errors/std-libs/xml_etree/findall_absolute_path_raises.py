# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "findall_absolute_path_raises"
# subject = "xml.etree.ElementTree.Element.findall"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.findall: findall_absolute_path_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.fromstring("<a><b/></a>").findall("/tag")
except SyntaxError:
    _raised = True
assert _raised, "findall_absolute_path_raises: expected SyntaxError"
print("findall_absolute_path_raises OK")
