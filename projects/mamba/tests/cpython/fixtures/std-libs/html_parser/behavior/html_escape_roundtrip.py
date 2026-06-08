# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "html_escape_roundtrip"
# subject = "html.escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.escape: html.escape('<b>x</b>') == '&lt;b&gt;x&lt;/b&gt;' and html.unescape round-trips it back"""
import html


escaped = html.escape("<b>x</b>")
assert escaped == "&lt;b&gt;x&lt;/b&gt;", escaped
assert html.unescape(escaped) == "<b>x</b>", html.unescape(escaped)

print("html_escape_roundtrip OK")
