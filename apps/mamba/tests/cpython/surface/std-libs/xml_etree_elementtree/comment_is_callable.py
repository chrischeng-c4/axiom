# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "comment_is_callable"
# subject = "ET.Comment"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Comment: comment_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.Comment)
print("comment_is_callable OK")
