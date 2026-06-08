# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "nested_element_text_and_tail"
# subject = "xml.etree.ElementTree.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element: for <outer>before<inner>inside</inner>after</outer>, outer.text is 'before', inner.text is 'inside', inner.tail is 'after'"""
import xml.etree.ElementTree as ET

outer = ET.fromstring("<outer>before<inner>inside</inner>after</outer>")
assert outer.text == "before", f"outer text = {outer.text!r}"
inner = list(outer)[0]
assert inner.text == "inside", f"inner text = {inner.text!r}"
assert inner.tail == "after", f"inner tail = {inner.tail!r}"

print("nested_element_text_and_tail OK")
