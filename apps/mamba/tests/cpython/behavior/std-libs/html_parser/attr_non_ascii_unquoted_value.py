# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_non_ascii_unquoted_value"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: non-ASCII unquoted attribute values are kept verbatim (<img src=/foo/bar.png alt=中文>)"""
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

assert attrs_of("<img src=/foo/bar.png alt=\u4e2d\u6587>") == [
    ("src", "/foo/bar.png"), ("alt", "\u4e2d\u6587")
], attrs_of("<img src=/foo/bar.png alt=\u4e2d\u6587>")

print("attr_non_ascii_unquoted_value OK")
