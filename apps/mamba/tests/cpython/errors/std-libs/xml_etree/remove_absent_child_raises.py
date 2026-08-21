# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "remove_absent_child_raises"
# subject = "xml.etree.ElementTree.Element.remove"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.remove: remove_absent_child_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.Element("a").remove(ET.Element("ghost"))
except ValueError:
    _raised = True
assert _raised, "remove_absent_child_raises: expected ValueError"
print("remove_absent_child_raises OK")
