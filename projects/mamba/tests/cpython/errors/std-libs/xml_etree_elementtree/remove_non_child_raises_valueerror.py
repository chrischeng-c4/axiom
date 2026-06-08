# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "remove_non_child_raises_valueerror"
# subject = "ET.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: remove_non_child_raises_valueerror (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.Element("parent").remove(ET.Element("other"))
except ValueError:
    _raised = True
assert _raised, "remove_non_child_raises_valueerror: expected ValueError"
print("remove_non_child_raises_valueerror OK")
