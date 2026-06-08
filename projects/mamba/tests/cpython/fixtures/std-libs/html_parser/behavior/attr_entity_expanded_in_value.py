# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_entity_expanded_in_value"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: entity references inside an attribute value are expanded (&amp;&gt;&lt;&quot;&apos; -> &><"')"""
from html.parser import HTMLParser



class AttrParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.attrs = None

    def handle_starttag(self, tag, attrs):
        self.attrs = attrs


def attrs_of(source):
    p = AttrParser()
    p.feed(source)
    return p.attrs

assert attrs_of("<a b='&amp;&gt;&lt;&quot;&apos;'>") == [
    ("b", "&><\"'")
], attrs_of("<a b='&amp;&gt;&lt;&quot;&apos;'>")

print("attr_entity_expanded_in_value OK")
