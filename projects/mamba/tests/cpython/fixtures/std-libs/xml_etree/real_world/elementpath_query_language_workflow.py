# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "real_world"
# case = "elementpath_query_language_workflow"
# subject = "xml.etree.ElementTree.Element.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element.findall: an ElementPath workflow resolves relative/child paths, wildcards, attribute and text predicates, findtext defaults, numeric-position and last() predicates, namespace wildcards/explicit-URI/prefix-map queries, and iter(tag) descendant walks"""
from xml.etree.ElementTree import XML, fromstring


def names(seq):
    return [e.tag for e in seq]


SAMPLE = (
    "<body>"
    "  <tag class='a'>text</tag>"
    "  <tag class='b' />"
    "  <section>"
    "    <tag class='b' id='inner'>subtext</tag>"
    "  </section>"
    "</body>"
)
e = XML(SAMPLE)

# Relative path, ./prefix, and child path all resolve.
assert e.find("tag").tag == "tag", "find tag"
assert e.find("./tag").tag == "tag", "find ./tag"
assert e.find("section/tag").tag == "tag", "find section/tag"

# findall returns every match in document order; missing path -> [].
assert names(e.findall("tag")) == ["tag", "tag"], "findall tag"
assert names(e.findall("*")) == ["tag", "tag", "section"], "findall *"
assert e.findall("ghost") == [], "findall missing"
assert names(e.findall(".//tag")) == ["tag", "tag", "tag"], "findall .//tag"

# Attribute predicates: presence and value match / not-match.
assert names(e.findall(".//tag[@class]")) == ["tag", "tag", "tag"], "[@class]"
assert names(e.findall('.//tag[@class="a"]')) == ["tag"], '[@class="a"]'
assert names(e.findall('.//tag[@class!="a"]')) == ["tag", "tag"], '[@class!="a"]'
assert names(e.findall(".//tag[@id]")) == ["tag"], "[@id]"

# Text predicate: [.='value']. The not-equal form only matches tags that
# carry text (the text=None tag does not satisfy "!=").
assert names(e.findall(".//tag[.='subtext']")) == ["tag"], "[.='subtext']"
assert names(e.findall(".//tag[.!='subtext']")) == ["tag", "tag"], "[.!=...]"

# findtext returns text, default for missing.
assert e.findtext("./tag") == "text", "findtext ./tag"
assert e.findtext("section/tag") == "subtext", "findtext section/tag"
assert e.findtext("ghost") is None, "findtext missing -> None"
assert e.findtext("ghost", "DFLT") == "DFLT", "findtext default"

# Numeric position predicates and last().
linear = XML(
    "<body><tag class='a'/><tag class='b'/>"
    "<tag class='c'/><tag class='d'/></body>"
)
assert linear.find("./tag[1]").attrib["class"] == "a", "[1]"
assert linear.find("./tag[2]").attrib["class"] == "b", "[2]"
assert linear.find("./tag[last()]").attrib["class"] == "d", "[last()]"
assert linear.find("./tag[last()-1]").attrib["class"] == "c", "[last()-1]"

# Namespace wildcards and explicit URIs.
nsroot = fromstring(
    '<a xmlns:x="X" xmlns:y="Y">'
    "<x:b><c/></x:b><b/><c><x:b/><b/></c><y:b/></a>"
)
assert names(nsroot.findall("{*}b")) == ["{X}b", "b", "{Y}b"], "{*}b"
assert names(nsroot.findall("{X}*")) == ["{X}b"], "{X}*"
assert names(nsroot.findall("{}*")) == ["b", "c"], "{}* (no-namespace)"

# namespaces= maps a prefix to a URI for the query.
assert len(nsroot.findall(".//xx:b", namespaces={"xx": "X"})) == 2, "prefix map X"
assert len(nsroot.findall(".//xx:b", namespaces={"xx": "Y"})) == 1, "prefix map Y"

# iter(tag) yields every matching descendant; iter()/iter('*') yield all.
doc = XML(
    "<document><house><room>a</room><room>b</room></house>"
    "<shed>x</shed><house><room>c</room></house></document>"
)
assert names(doc.iter("room")) == ["room", "room", "room"], "iter('room')"
assert names(doc.iter("house")) == ["house", "house"], "iter('house')"
all_tags = ["document", "house", "room", "room", "shed", "house", "room"]
assert names(doc.iter()) == all_tags, "iter() walks all"
assert names(doc.iter("*")) == all_tags, "iter('*') walks all"

print("elementpath_query_language_workflow OK")
