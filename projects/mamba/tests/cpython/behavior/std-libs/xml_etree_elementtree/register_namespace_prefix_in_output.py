# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "register_namespace_prefix_in_output"
# subject = "ET.register_namespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.register_namespace: register_namespace maps a URI to a prefix so a Clark-notation '{uri}tag' element serializes with that prefix"""
import xml.etree.ElementTree as ET

ET.register_namespace("ns", "http://example.com/ns")
r = ET.Element("{http://example.com/ns}root")
ET.SubElement(r, "{http://example.com/ns}child").text = "data"
s = ET.tostring(r, encoding="unicode")
assert "ns:root" in s, f"registered prefix in output: {s!r}"
assert "ns:child" in s, f"registered prefix on child: {s!r}"

print("register_namespace_prefix_in_output OK")
