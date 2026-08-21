# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "incremental_chunked_feed"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: feeding <div><span>hello</span></div> in 3-char chunks then close() yields the same div/span tags and 'hello' data"""
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
html = "<div><span>hello</span></div>"
for i in range(0, len(html), 3):
    p.feed(html[i:i + 3])
p.close()
starts = [e[1] for e in p.events if e[0] == "start"]
assert "div" in starts and "span" in starts, starts
text = "".join(e[1] for e in p.events if e[0] == "data")
assert "hello" in text, text

print("incremental_chunked_feed OK")
