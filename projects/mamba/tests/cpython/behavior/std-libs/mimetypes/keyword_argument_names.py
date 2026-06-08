# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "keyword_argument_names"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: the public keyword-argument names url=/type=/strict= are part of the API: guess_type(url=, strict=), guess_extension(type=, strict=), guess_all_extensions(type=, strict=)"""
import mimetypes

db = mimetypes.MimeTypes()

# Keyword-argument names: url=, type=, strict=.
assert db.guess_type(url="foo.html", strict=True) == ("text/html", None), "kw url"
assert db.guess_all_extensions(type="image/jpg", strict=True) == [], "kw type strict"
assert db.guess_extension(type="image/jpg", strict=False) == ".jpg", "kw type loose"
print("keyword_argument_names OK")
