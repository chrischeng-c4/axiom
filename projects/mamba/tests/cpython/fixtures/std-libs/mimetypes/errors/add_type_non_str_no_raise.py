# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "errors"
# case = "add_type_non_str_no_raise"
# subject = "mimetypes.add_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.add_type: add_type(non-str, ext) does NOT raise under CPython 3.12 (the mapping is forgiving); it silently registers a non-string type"""
import mimetypes

# A fresh registry keeps the global table clean.
db = mimetypes.MimeTypes()
_raised = False
try:
    db.add_type(123, ".zzqq")  # type: ignore[arg-type]
except (TypeError, AttributeError):
    _raised = True
assert not _raised, "add_type(non-str, ext) must not raise"
# The non-string type is silently registered and echoed back verbatim.
assert db.guess_type("a.zzqq") == (123, None), f"guess = {db.guess_type('a.zzqq')!r}"
print("add_type_non_str_no_raise OK")
