# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "pathname2url_roundtrip"
# subject = "urllib.request.pathname2url"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.request.pathname2url: pathname2url / url2pathname round-trip a relative path and percent-escape then recover reserved chars in a path component"""
from urllib.request import pathname2url, url2pathname

import os
from urllib.parse import quote

rel = os.path.join("parts", "of", "a", "path")
url = pathname2url(rel)
assert url == "parts/of/a/path", f"pathname2url = {url!r}"
assert url2pathname(url) == rel, "url2pathname round-trip"
needs = os.path.join("needs", "quot=ing", "here")
escaped = pathname2url(needs)
assert escaped == "needs/%s/here" % quote("quot=ing"), \
    f"pathname2url quoting = {escaped!r}"
assert url2pathname(escaped) == needs, "quoting round-trip"

print("pathname2url_roundtrip OK")
