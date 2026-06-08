# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "behavior"
# case = "iterparse_streams_start_end_events"
# subject = "ET.iterparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.iterparse: iterparse(src, events=('start','end')) streams (event, element) pairs in document order, start before each subtree and end after it"""
import xml.etree.ElementTree as ET

import io
src = io.StringIO("<a><b/><c/></a>")
events = [(ev, el.tag) for ev, el in ET.iterparse(src, events=("start", "end"))]
assert events == [
    ("start", "a"),
    ("start", "b"),
    ("end", "b"),
    ("start", "c"),
    ("end", "c"),
    ("end", "a"),
], f"iterparse events = {events!r}"

print("iterparse_streams_start_end_events OK")
