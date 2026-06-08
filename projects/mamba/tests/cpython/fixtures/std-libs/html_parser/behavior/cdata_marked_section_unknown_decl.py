# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "cdata_marked_section_unknown_decl"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a CDATA-looking marked section inside foreign content fires unknown_decl with 'CDATA[raw'"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of('<svg><text y="100"><![CDATA[raw]]></text></svg>') == [
    ("starttag", "svg", []),
    ("starttag", "text", [("y", "100")]),
    ("unknown decl", "CDATA[raw"),
    ("endtag", "text"),
    ("endtag", "svg"),
], events_of('<svg><text y="100"><![CDATA[raw]]></text></svg>')

print("cdata_marked_section_unknown_decl OK")
