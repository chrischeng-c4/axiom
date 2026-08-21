# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "tag_names_lowercased"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: tag names are lowercased (HTML is case-insensitive); <DIV><P>...</P></DIV> reports div/p in all start and end events"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

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

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
p.feed("<DIV><P>text</P></DIV>")
names = [e[1] for e in p.events if e[0] in ("start", "end")]
assert all(t == t.lower() for t in names), names
assert "div" in names and "p" in names, names

print("tag_names_lowercased OK")
