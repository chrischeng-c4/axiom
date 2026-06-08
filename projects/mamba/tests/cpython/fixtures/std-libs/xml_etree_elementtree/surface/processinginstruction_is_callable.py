# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "processinginstruction_is_callable"
# subject = "ET.ProcessingInstruction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.ProcessingInstruction: processinginstruction_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.ProcessingInstruction)
print("processinginstruction_is_callable OK")
