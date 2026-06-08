# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "subclass_handler_exception_propagates"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: an exception raised inside a subclass handle_starttag override propagates out of feed() (ValueError on a <script> tag)"""
from html.parser import HTMLParser


class StrictParser(HTMLParser):
    def handle_starttag(self, tag, attrs):
        if tag == "script":
            raise ValueError("scripts not allowed")


_raised = False
try:
    StrictParser().feed("<html><body><script>bad</script></body></html>")
except ValueError:
    _raised = True
assert _raised, "subclass handle_starttag ValueError must propagate out of feed()"
print("subclass_handler_exception_propagates OK")
