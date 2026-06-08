# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "comment_and_pi_tag_identity"
# subject = "ET.Comment"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Comment: Comment(text) and ProcessingInstruction(target, text) produce nodes whose .tag is the factory itself and whose text is preserved"""
import xml.etree.ElementTree as ET

comment = ET.Comment("note")
assert comment.tag is ET.Comment, "comment tag identity"
assert comment.text == "note", f"comment text = {comment.text!r}"
pi = ET.ProcessingInstruction("target", "value")
assert pi.tag is ET.ProcessingInstruction, "pi tag identity"

print("comment_and_pi_tag_identity OK")
