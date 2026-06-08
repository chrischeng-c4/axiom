# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "malformed_input_does_not_raise"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: the default parser is forgiving: feeding an unclosed tag (<html><body><unclosed_tag>text) records what it can and does NOT raise"""
from html.parser import HTMLParser


class CaptureParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.tags = []

    def handle_starttag(self, tag, attrs):
        self.tags.append(tag)


cp = CaptureParser()
# A forgiving lexer: this must not raise; it records what start tags it can.
cp.feed("<html><body><unclosed_tag>text")
assert cp.tags == ["html", "body", "unclosed_tag"], cp.tags
print("malformed_input_does_not_raise OK")
