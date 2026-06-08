# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "surface"
# case = "api_processing_instruction_is_present"
# subject = "xml.dom.minidom.ProcessingInstruction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.minidom.ProcessingInstruction: api_processing_instruction_is_present (surface)."""
import xml.dom.minidom

assert hasattr(xml.dom.minidom, "ProcessingInstruction")
print("api_processing_instruction_is_present OK")
