# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "real_world"
# case = "file_parse_write_iterparse_roundtrip"
# subject = "xml.etree.ElementTree.parse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.parse: a file I/O workflow (all state inside a TemporaryDirectory) parses a file, writes it back and re-reads to confirm byte round-trip, checks default-numeric vs utf-8-raw write encoding, drives iterparse end and start/end event streams, and parses from an in-memory binary stream"""
import io
import os
import tempfile
from xml.etree.ElementTree import ElementTree, XML, parse, iterparse, tostring


with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "in.xml")
    with open(src, "w", encoding="utf-8") as f:
        f.write('<root><element key="value">text</element><empty-element/></root>')

    # parse() reads a file path into a tree.
    tree = parse(src)
    root = tree.getroot()
    assert root.tag == "root", f"parsed root = {root.tag!r}"
    assert [c.tag for c in root] == ["element", "empty-element"], \
        f"parsed children = {[c.tag for c in root]!r}"
    assert root.find("element").get("key") == "value", "parsed attribute"

    # tree.write() back to a path, then re-read to confirm round-trip.
    out = os.path.join(d, "out.xml")
    tree.write(out)
    reread = parse(out).getroot()
    assert tostring(reread) == tostring(root), "round-trip via file mismatch"

    # Default write encoding numeric-escapes non-ASCII.
    ElementTree(XML("<site>\xf8</site>")).write(out)
    with open(out, "rb") as f:
        assert f.read() == b"<site>&#248;</site>", "default-encoded write"

    # encoding='utf-8' emits raw UTF-8 bytes (no numeric escape, no decl).
    ElementTree(XML("<site>\xf8</site>")).write(out, encoding="utf-8")
    with open(out, "rb") as f:
        assert f.read() == b"<site>\xc3\xb8</site>", "utf-8 write"

    # iterparse over a file yields ('end', tag) events in document order.
    events = [(action, elem.tag) for action, elem in iterparse(src)]
    assert events == [
        ("end", "element"),
        ("end", "empty-element"),
        ("end", "root"),
    ], f"iterparse end events = {events!r}"

    # iterparse with explicit start/end events.
    se = [(action, elem.tag) for action, elem in iterparse(src, ("start", "end"))]
    assert se == [
        ("start", "root"),
        ("start", "element"),
        ("end", "element"),
        ("start", "empty-element"),
        ("end", "empty-element"),
        ("end", "root"),
    ], f"iterparse start/end = {se!r}"

# Parsing from an in-memory binary stream (no real file needed).
stream = io.BytesIO(b"<doc><a>1</a><b>2</b></doc>")
mem_tree = ElementTree(file=stream)
assert mem_tree.find("a").text == "1", "stream parse a"
assert mem_tree.find("b").text == "2", "stream parse b"

print("file_parse_write_iterparse_roundtrip OK")
