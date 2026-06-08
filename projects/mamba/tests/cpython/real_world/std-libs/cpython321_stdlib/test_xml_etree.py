# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_xml_etree"
# subject = "cpython321.test_xml_etree"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_xml_etree.py"
# status = "filled"
# ///
"""cpython321.test_xml_etree: execute CPython 3.12 seed test_xml_etree"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: xml.etree.ElementTree — Element / SubElement construction,
# tostring() of an empty element / nested children / repeated siblings,
# fromstring() returning a root with a .tag attribute.
# Intentionally NOT exercised on mamba today (tracked separately):
#   * Element.text setter (assignment is a no-op — read-back is None)
#   * Element.find / .findall / .set / .get (AttributeError on mamba's dict
#     stub objects)
#   * iter() of an Element (returns the dict keys, not the children)
from xml.etree.ElementTree import Element, SubElement, tostring, fromstring

_ledger: list[int] = []

# An empty Element renders as a self-closing tag
_root = Element("root")
assert tostring(_root) == b"<root />", (
    f"tostring(Element('root')) == '<root />', got {tostring(_root)!r}"
)
_ledger.append(1)

# Element exposes its tag
assert _root.tag == "root", f"Element('root').tag == 'root', got {_root.tag!r}"
_ledger.append(1)

# SubElement nests a child under the parent
_parent = Element("parent")
SubElement(_parent, "child")
assert tostring(_parent) == b"<parent><child /></parent>", (
    f"tostring nested == '<parent><child /></parent>', got {tostring(_parent)!r}"
)
_ledger.append(1)

# Multiple SubElement calls preserve insertion order
_p = Element("p")
SubElement(_p, "a")
SubElement(_p, "b")
SubElement(_p, "c")
assert tostring(_p) == b"<p><a /><b /><c /></p>", (
    f"tostring 3 ordered children == '<p><a /><b /><c /></p>', got {tostring(_p)!r}"
)
_ledger.append(1)

# A SubElement's tag is what was passed in
_q = Element("q")
_sub = SubElement(_q, "leaf")
assert _sub.tag == "leaf", f"SubElement(..., 'leaf').tag == 'leaf', got {_sub.tag!r}"
_ledger.append(1)

# fromstring() of a single-tag document yields an Element with that tag
_doc = fromstring("<doc/>")
assert _doc.tag == "doc", (
    f"fromstring('<doc/>').tag == 'doc', got {_doc.tag!r}"
)
_ledger.append(1)

# fromstring() of a document with text/children — tag still resolves
_d2 = fromstring("<a><b/></a>")
assert _d2.tag == "a", f"fromstring('<a><b/></a>').tag == 'a', got {_d2.tag!r}"
_ledger.append(1)

# A differently-named root rounds through fromstring distinctly
_d3 = fromstring("<other_root/>")
assert _d3.tag == "other_root", (
    f"fromstring('<other_root/>').tag == 'other_root', got {_d3.tag!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_xml_etree {sum(_ledger)} asserts")
