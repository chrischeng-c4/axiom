# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_mixed_quoting_and_valueless"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: single, double, and unquoted values parse equally and a valueless flag attribute gets value None"""
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

assert attrs_of("<a b='v' c=\"v\" d=v e>") == [
    ("b", "v"), ("c", "v"), ("d", "v"), ("e", None)
], attrs_of("<a b='v' c=\"v\" d=v e>")

print("attr_mixed_quoting_and_valueless OK")
