# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_url_only_extension"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: URL-style inputs are matched by trailing extension only: http://example.com/path/file.json?q=1 -> application/json"""
import mimetypes

t, _ = mimetypes.guess_type("http://example.com/path/file.json?q=1")
assert t == "application/json", f"url json = {t!r}"
print("guess_type_url_only_extension OK")
