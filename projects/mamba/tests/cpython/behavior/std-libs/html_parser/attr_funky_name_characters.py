# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_funky_name_characters"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: funky-but-legal attribute name characters (a.b c:d e-f) survive intact"""
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

assert attrs_of("<a a.b='v' c:d=v e-f=v>") == [
    ("a.b", "v"), ("c:d", "v"), ("e-f", "v")
], attrs_of("<a a.b='v' c:d=v e-f=v>")

print("attr_funky_name_characters OK")
