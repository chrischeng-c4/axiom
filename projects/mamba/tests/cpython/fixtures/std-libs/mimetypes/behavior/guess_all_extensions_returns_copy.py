# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_all_extensions_returns_copy"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: guess_all_extensions returns a fresh list copy: mutating the result cannot corrupt the internal table"""
import mimetypes

db = mimetypes.MimeTypes()
db.add_type("test-type", ".strict-ext")

# The returned list is a fresh copy; mutating it cannot corrupt the table.
got = db.guess_all_extensions("test-type")
got.append(".no-such-ext")
assert ".no-such-ext" not in db.guess_all_extensions("test-type"), "copy isolated"
print("guess_all_extensions_returns_copy OK")
