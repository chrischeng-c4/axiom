# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "real_world"
# case = "extract_links_from_document"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a link-extractor HTMLParser subclass collects every href from anchor tags in a small HTML document"""
from html.parser import HTMLParser



class LinkExtractor(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        if tag == "a":
            for name, value in attrs:
                if name == "href" and value is not None:
                    self.links.append(value)


DOC = """
<html><body>
  <h1>Links</h1>
  <p>See <a href="https://example.com/one">one</a> and
     <a href="/relative/two">two</a>.</p>
  <a href="https://example.com/three" class="cta">three</a>
  <a name="anchor">no href</a>
</body></html>
"""

ext = LinkExtractor()
ext.feed(DOC)
assert ext.links == [
    "https://example.com/one",
    "/relative/two",
    "https://example.com/three",
], ext.links

print("extract_links_from_document OK")
