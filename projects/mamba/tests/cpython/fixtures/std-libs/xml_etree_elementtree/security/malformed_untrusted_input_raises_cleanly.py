# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "security"
# case = "malformed_untrusted_input_raises_cleanly"
# subject = "ET.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: a battery of malformed untrusted documents (unclosed tag, mismatched end tag, bare ampersand, junk after root) each raise ParseError cleanly rather than crashing or silently truncating the parse"""
import xml.etree.ElementTree as ET

# Each malformed document must raise ParseError -- a clean, catchable failure
# rather than a crash or a silently truncated parse.
malformed = [
    "<unclosed>",                 # never-closed tag
    "<a></b>",                    # mismatched end tag
    "<a>bare & ampersand</a>",    # unescaped ampersand
    "<a/>trailing junk",          # content after the root element
    "<a><b></a>",                 # improperly nested
]
for doc in malformed:
    _raised = False
    try:
        ET.fromstring(doc)
    except ET.ParseError:
        _raised = True
    assert _raised, f"expected ParseError for {doc!r}"

print("malformed_untrusted_input_raises_cleanly OK")
