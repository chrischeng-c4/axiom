# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "starttag_reports_tag_and_attrs"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: handle_starttag reports the tag name and an attr list; <a href=... id=...> yields tag 'a' with both attributes"""
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
p.feed('<a href="http://example.com" id="link">text</a>')
starts = [e for e in p.events if e[0] == "start"]
assert len(starts) == 1, starts
assert starts[0][1] == "a", starts[0][1]
attrs = dict(starts[0][2])
assert attrs["href"] == "http://example.com", attrs["href"]
assert attrs["id"] == "link", attrs["id"]

print("starttag_reports_tag_and_attrs OK")
