# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "surface"
# case = "import_element_tree"
# subject = "xml.etree.ElementTree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""xml.etree.ElementTree: import_element_tree (surface)."""
import xml.etree.ElementTree

assert hasattr(xml.etree.ElementTree, "fromstring")
print("import_element_tree OK")
