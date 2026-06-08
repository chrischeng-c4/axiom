# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "security"
# case = "external_entity_not_fetched"
# subject = "ET.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: an XXE-style document declaring an external SYSTEM entity pointing at a local file never fetches or inlines that file; the reference is treated as an undefined entity and raises ParseError, so no external resource is read"""
import xml.etree.ElementTree as ET

# An XXE-style payload: an external SYSTEM entity that, if expanded, would
# read a local file. ElementTree's expat parser does not fetch external
# entities, so the reference is left undefined and parsing fails with
# ParseError -- no file is ever opened.
payload = (
    '<?xml version="1.0"?>'
    '<!DOCTYPE foo [ <!ENTITY xxe SYSTEM "file:///etc/passwd"> ]>'
    "<foo>&xxe;</foo>"
)
_raised = False
try:
    ET.fromstring(payload)
except ET.ParseError as e:
    _raised = True
    assert "undefined entity" in str(e), f"reason: {e}"
assert _raised, "external SYSTEM entity must raise ParseError (not fetch the file)"

print("external_entity_not_fetched OK")
