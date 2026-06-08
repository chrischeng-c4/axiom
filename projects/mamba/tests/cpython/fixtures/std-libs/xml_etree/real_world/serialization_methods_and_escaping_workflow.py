# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "real_world"
# case = "serialization_methods_and_escaping_workflow"
# subject = "xml.etree.ElementTree.tostring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.tostring: a serializer workflow covers method=xml/html/text, text vs attribute entity escaping, us-ascii numeric vs utf-8 raw byte encoding, whitespace-control attr escaping, attribute insertion order, ProcessingInstruction/PI/Comment delimiters, and indent() default/tab pretty-printing"""
import io
from xml.etree.ElementTree import (
    Element,
    XML,
    ElementTree,
    ProcessingInstruction,
    PI,
    Comment,
    indent,
    tostring,
)


def serialize(elem, **opts):
    enc = opts.setdefault("encoding", "unicode")
    buf = io.StringIO() if enc == "unicode" else io.BytesIO()
    ElementTree(elem).write(buf, **opts)
    return buf.getvalue()


# method="xml" (default) keeps void elements self-closing and escapes entities.
e = XML("<html><link/><script>1 &lt; 2</script></html>")
assert serialize(e) == "<html><link /><script>1 &lt; 2</script></html>", \
    f"xml = {serialize(e)!r}"

# method="html" emits HTML void elements unclosed and does NOT escape <,>.
assert serialize(e, method="html") == (
    "<html><link><script>1 < 2</script></html>"
), f"html = {serialize(e, method='html')!r}"

# method="text" emits text content only.
assert serialize(e, method="text") == "1 < 2", f"text = {serialize(e, method='text')!r}"

# Special characters in text are escaped (but ' and " stay as-is in text).
t = Element("tag")
t.text = "<&\"'>"
assert serialize(t) == "<tag>&lt;&amp;\"'&gt;</tag>", f"text escape = {serialize(t)!r}"

# In attribute values, the double quote is also escaped.
a = Element("tag")
a.attrib["key"] = "<&\"'>"
assert serialize(a) == '<tag key="&lt;&amp;&quot;\'&gt;" />', \
    f"attr escape = {serialize(a)!r}"

# us-ascii encoding numeric-escapes non-ASCII; utf-8 emits raw bytes.
u = Element("tag")
u.text = "\xe5\xf6\xf6<>"
assert serialize(u, encoding="us-ascii") == b"<tag>&#229;&#246;&#246;&lt;&gt;</tag>", \
    "us-ascii numeric escape"
assert serialize(u, encoding="utf-8") == b"<tag>\xc3\xa5\xc3\xb6\xc3\xb6&lt;&gt;</tag>", \
    "utf-8 raw bytes"

# Whitespace control chars in attributes are numeric-escaped.
w = Element("test")
w.set("a", "\r")
w.set("c", "\t\n\r ")
assert tostring(w) == b'<test a="&#13;" c="&#09;&#10;&#13; " />', \
    f"whitespace attr = {tostring(w)!r}"

# Attribute insertion order is preserved in output.
order = Element("c", status="public", company="example")
assert tostring(order) == b'<c status="public" company="example" />', \
    f"attr order = {tostring(order)!r}"

# ProcessingInstruction / PI serialize with ?...? delimiters.
assert tostring(ProcessingInstruction("test", "instruction")) == b"<?test instruction?>", \
    "PI serialize"
assert tostring(PI("test", "instruction")) == b"<?test instruction?>", "PI alias"

# Comment serializes with <!-- --> delimiters.
assert tostring(Comment("hello")) == b"<!--hello-->", "Comment serialize"

# indent() pretty-prints in place with 2-space default.
doc = XML("<html><body>text</body></html>")
indent(doc)
assert tostring(doc) == b"<html>\n  <body>text</body>\n</html>", \
    f"indent default = {tostring(doc)!r}"

# indent(space=...) honors the custom indent string.
doc2 = XML("<html><body><p>pre<br/>post</p><p>text</p></body></html>")
indent(doc2, space="\t")
assert tostring(doc2) == (
    b"<html>\n\t<body>\n\t\t<p>pre<br />post</p>\n\t\t<p>text</p>\n\t</body>\n</html>"
), f"indent tab = {tostring(doc2)!r}"

print("serialization_methods_and_escaping_workflow OK")
