# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "findtext_missing_returns_default"
# subject = "xml.etree.ElementTree.Element.findtext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.findtext: Element.findtext on a missing path returns the supplied default value, not None"""
import xml.etree.ElementTree as ET

root = ET.fromstring("<a><b/></a>")
assert root.findtext("missing", "X") == "X", "findtext missing must return the default"
assert root.findtext("missing") is None, "findtext missing without default is None"

print("findtext_missing_returns_default OK")
