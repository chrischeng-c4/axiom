# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "real_world"
# case = "namespace_qname_and_prefix_workflow"
# subject = "xml.etree.ElementTree.QName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.QName: a namespaced-document workflow exercises QName str/equality, {uri}tag auto-prefix (ns0) on tags and attributes, default_namespace prefix dropping, a second-namespace prefix under default_namespace, and namespaced findall with {uri}tag and .// paths"""
import io
from xml.etree.ElementTree import Element, SubElement, QName, XML, ElementTree, tostring


def serialize(elem, **opts):
    buf = io.StringIO()
    ElementTree(elem).write(buf, encoding="unicode", **opts)
    return buf.getvalue()


# QName str() always produces the {uri}local form.
assert str(QName("ns", "tag")) == "{ns}tag", "QName(uri, local)"
assert str(QName("{ns}tag")) == "{ns}tag", "QName({uri}local)"

# QName equality compares the {uri}local string.
assert QName("ns", "tag") == QName("ns", "tag"), "QName equal"
assert QName("ns", "tag") != QName("ns", "other"), "QName not equal"
assert QName("ns", "tag") == "{ns}tag", "QName == string"
assert QName("ns", "tag") != "ns:tag", "QName != prefixed string"

# A {uri}tag element serializes with an auto-generated ns0 prefix.
elem = Element("{uri}tag")
assert serialize(elem) == '<ns0:tag xmlns:ns0="uri" />', f"got {serialize(elem)!r}"

# QName tags behave identically to {uri}tag strings.
qelem = Element(QName("uri", "tag"))
SubElement(qelem, QName("uri", "tag1"))
SubElement(qelem, QName("uri", "tag2"))
assert serialize(qelem) == (
    '<ns0:tag xmlns:ns0="uri"><ns0:tag1 /><ns0:tag2 /></ns0:tag>'
), f"got {serialize(qelem)!r}"

# Namespaced attributes are prefixed too.
attr_elem = Element("{uri}tag")
attr_elem.attrib["{uri}key"] = "value"
assert serialize(attr_elem) == (
    '<ns0:tag xmlns:ns0="uri" ns0:key="value" />'
), f"got {serialize(attr_elem)!r}"

# default_namespace drops the prefix for matching elements.
e = Element("{default}elem")
SubElement(e, "{default}elem")
assert serialize(e, default_namespace="default") == (
    '<elem xmlns="default"><elem /></elem>'
), f"got {serialize(e, default_namespace='default')!r}"

# A second, different namespace still gets a prefix under default_namespace.
e2 = Element("{default}elem")
SubElement(e2, "{default}elem")
SubElement(e2, "{other}elem")
assert serialize(e2, default_namespace="default") == (
    '<elem xmlns="default" xmlns:ns1="other"><elem /><ns1:elem /></elem>'
), f"got {serialize(e2, default_namespace='default')!r}"

# Parsing a namespaced doc, then finding by {uri}tag.
nsdoc = XML(
    '<body xmlns="http://effbot.org/ns">'
    "<tag>text</tag><tag/><section><tag>sub</tag></section></body>"
)
assert nsdoc.findall("tag") == [], "bare tag does not match namespaced"
matched = nsdoc.findall("{http://effbot.org/ns}tag")
assert len(matched) == 2, f"namespaced findall = {len(matched)!r}"
deep = nsdoc.findall(".//{http://effbot.org/ns}tag")
assert len(deep) == 3, f"deep namespaced findall = {len(deep)!r}"

# Well-known namespaces serialize with their conventional prefix.
xmllang = XML("<tag xml:lang='en' />")
assert tostring(xmllang, encoding="unicode") == '<tag xml:lang="en" />', \
    "xml:lang preserved"

print("namespace_qname_and_prefix_workflow OK")
