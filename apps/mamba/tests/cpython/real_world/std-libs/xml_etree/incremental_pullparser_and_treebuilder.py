# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "real_world"
# case = "incremental_pullparser_and_treebuilder"
# subject = "xml.etree.ElementTree.XMLPullParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.XMLPullParser: an incremental-parsing workflow feeds XMLPullParser in chunks (end-only, start/end, namespaced, and comment events), builds elements via TreeBuilder start/data/end (flat and nested), and resolves a custom entity registered on an XMLParser"""
from xml.etree.ElementTree import XMLPullParser, TreeBuilder, XMLParser, tostring


def end_tags(parser):
    return [(action, elem.tag) for action, elem in parser.read_events()]


# Default XMLPullParser reports only 'end' events, fed in chunks.
p = XMLPullParser()
p.feed("<root>\n  <element key='value'>text</element>\n")
assert end_tags(p) == [("end", "element")], "first end event"
p.feed("<empty-element/>\n")
assert end_tags(p) == [("end", "empty-element")], "second end event"
p.feed("</root>\n")
assert end_tags(p) == [("end", "root")], "root end event"
assert p.close() is None, "close returns None"

# read_events drains only the events seen so far; later feeds add more.
p2 = XMLPullParser(events=("start", "end"))
p2.feed("<root>")
assert end_tags(p2) == [("start", "root")], "start event before children"
p2.feed("<a/></root>")
assert end_tags(p2) == [
    ("start", "a"),
    ("end", "a"),
    ("end", "root"),
], "remaining start/end events"

# A namespaced default scope expands tags in events.
p3 = XMLPullParser(events=("start", "end"))
p3.feed("<element xmlns='foo'><child/></element>")
assert end_tags(p3) == [
    ("start", "{foo}element"),
    ("start", "{foo}child"),
    ("end", "{foo}child"),
    ("end", "{foo}element"),
], "namespaced events"

# comment events carry the comment text.
p4 = XMLPullParser(events=("comment",))
p4.feed("<!-- note --><root/>")
comments = [(action, elem.text) for action, elem in p4.read_events()]
assert comments == [("comment", " note ")], f"comment events = {comments!r}"

# TreeBuilder builds an element from explicit start/data/end calls.
b = TreeBuilder()
b.start("tag", {})
b.data("hello")
b.end("tag")
built = b.close()
assert tostring(built) == b"<tag>hello</tag>", f"treebuilder = {tostring(built)!r}"

# TreeBuilder nesting produces a child tree.
b2 = TreeBuilder()
b2.start("a", {})
b2.start("b", {"k": "v"})
b2.data("inner")
b2.end("b")
b2.end("a")
nested = b2.close()
assert tostring(nested) == b'<a><b k="v">inner</b></a>', \
    f"nested treebuilder = {tostring(nested)!r}"

# A custom entity can be registered on an XMLParser before feeding.
entity_xml = (
    "<!DOCTYPE points ["
    "<!ENTITY % user-entities SYSTEM 'user-entities.xml'>"
    "%user-entities;"
    "]><document>&entity;</document>"
)
parser = XMLParser()
parser.entity["entity"] = "text"
parser.feed(entity_xml)
resolved = parser.close()
assert tostring(resolved) == b"<document>text</document>", \
    f"custom entity = {tostring(resolved)!r}"

print("incremental_pullparser_and_treebuilder OK")
