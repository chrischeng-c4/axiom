# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "indent_pretty_prints_two_space_nesting"
# subject = "ET.indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.indent: indent(tree) rewrites text/tail so tostring pretty-prints with newlines and two-space-per-level nesting"""
import xml.etree.ElementTree as ET

tree = ET.Element("root")
inner = ET.SubElement(tree, "a")
ET.SubElement(inner, "b")
ET.indent(tree)
out = ET.tostring(tree, encoding="unicode")
assert out == "<root>\n  <a>\n    <b />\n  </a>\n</root>", f"indent = {out!r}"

print("indent_pretty_prints_two_space_nesting OK")
