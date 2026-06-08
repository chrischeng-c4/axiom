# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_case_insensitive"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: extension matching is case-insensitive: FILE.HTML and file.html both map to text/html"""
import mimetypes

ta, _ = mimetypes.guess_type("FILE.HTML")
tb, _ = mimetypes.guess_type("file.html")
assert ta == "text/html", f"uppercase ext: {ta!r}"
assert tb == "text/html", f"lowercase ext: {tb!r}"
assert ta == tb, "case-folded to the same type"
print("guess_type_case_insensitive OK")
