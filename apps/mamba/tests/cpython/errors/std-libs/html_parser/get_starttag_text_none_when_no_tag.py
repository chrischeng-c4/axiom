# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "get_starttag_text_none_when_no_tag"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: get_starttag_text() returns None on a fresh parser with no current start tag"""
from html.parser import HTMLParser

p = HTMLParser()
assert p.get_starttag_text() is None, p.get_starttag_text()
print("get_starttag_text_none_when_no_tag OK")
