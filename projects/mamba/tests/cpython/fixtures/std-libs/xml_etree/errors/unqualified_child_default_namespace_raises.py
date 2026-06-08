# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "errors"
# case = "unqualified_child_default_namespace_raises"
# subject = "xml.etree.ElementTree.ElementTree.write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.ElementTree.write: serializing a default_namespace doc that contains an unqualified child raises ValueError"""
import io
import xml.etree.ElementTree as ET

bad = ET.Element("{default}elem")
ET.SubElement(bad, "plain")

_raised = False
try:
    buf = io.StringIO()
    ET.ElementTree(bad).write(buf, encoding="unicode", default_namespace="default")
except ValueError:
    _raised = True
assert _raised, "unqualified child + default_namespace must raise ValueError"

print("unqualified_child_default_namespace_raises OK")
