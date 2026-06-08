# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "convert_charrefs_false_bad_entity_no_raise"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: with convert_charrefs=False, feeding a non-entity (&not_an_entity;) does not raise"""
from html.parser import HTMLParser

p = HTMLParser(convert_charrefs=False)
# A non-entity ampersand sequence must be tolerated, not raised on.
p.feed("&not_an_entity;")
p.close()
print("convert_charrefs_false_bad_entity_no_raise OK")
