# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html"
# dimension = "behavior"
# case = "html_tests__test_escape"
# subject = "cpython.test_html.HtmlTests.test_escape"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_html.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_html.py::HtmlTests::test_escape
"""Auto-ported test: HtmlTests::test_escape (CPython 3.12 oracle)."""


import html
import unittest


'\nTests for the html module functions.\n'


# --- test body ---

assert html.escape('\'<script>"&foo;"</script>\'') == '&#x27;&lt;script&gt;&quot;&amp;foo;&quot;&lt;/script&gt;&#x27;'

assert html.escape('\'<script>"&foo;"</script>\'', False) == '\'&lt;script&gt;"&amp;foo;"&lt;/script&gt;\''
print("HtmlTests::test_escape: ok")
