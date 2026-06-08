# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "treebuilder_builds_tree_imperatively"
# subject = "ET.TreeBuilder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.TreeBuilder: TreeBuilder.start/data/end/close builds an element tree imperatively, carrying tag, attributes, and text"""
import xml.etree.ElementTree as ET

builder = ET.TreeBuilder()
builder.start("a", {"k": "v"})
builder.data("text")
builder.end("a")
built = builder.close()
assert built.tag == "a", "builder tag"
assert built.get("k") == "v", "builder attr"
assert built.text == "text", "builder data"

print("treebuilder_builds_tree_imperatively OK")
