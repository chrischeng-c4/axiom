# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_adjacent_no_space"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: adjacent attributes with no separating space still split (<a width="100%"cellspacing=0>)"""
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

assert attrs_of('<a width="100%"cellspacing=0>') == [
    ("width", "100%"), ("cellspacing", "0")
], attrs_of('<a width="100%"cellspacing=0>')

print("attr_adjacent_no_space OK")
