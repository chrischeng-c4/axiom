# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "reset_then_refeed"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: reset() returns the parser to a clean state so a subsequent feed of <div>new</div> is parsed correctly"""
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
p.feed("<span>text</span>")
p.reset()
p.events.clear()
p.feed("<div>new</div>")
starts = [e[1] for e in p.events if e[0] == "start"]
assert "div" in starts, p.events
data = [e[1] for e in p.events if e[0] == "data"]
assert "new" in data, p.events

print("reset_then_refeed OK")
