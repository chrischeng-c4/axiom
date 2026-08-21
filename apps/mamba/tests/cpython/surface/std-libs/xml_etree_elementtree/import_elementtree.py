# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "import_elementtree"
# subject = "ET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET: import_elementtree (surface)."""
import xml.etree.ElementTree as ET

assert hasattr(ET, "Element")
print("import_elementtree OK")
