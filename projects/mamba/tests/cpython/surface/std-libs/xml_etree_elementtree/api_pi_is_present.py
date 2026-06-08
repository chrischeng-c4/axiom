# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "api_pi_is_present"
# subject = "xml.etree.ElementTree.PI"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.etree.ElementTree.PI: api_pi_is_present (surface)."""
import xml.etree.ElementTree

assert hasattr(xml.etree.ElementTree, "PI")
print("api_pi_is_present OK")
