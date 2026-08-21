# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "register_namespace_is_callable"
# subject = "ET.register_namespace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.register_namespace: register_namespace_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.register_namespace)
print("register_namespace_is_callable OK")
