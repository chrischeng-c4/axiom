# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_all_extensions_list"
# subject = "mimetypes.guess_all_extensions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_all_extensions: guess_all_extensions returns the full list of registered extensions for a type: text/html yields .html and .htm"""
import mimetypes

exts = mimetypes.guess_all_extensions("text/html")
assert isinstance(exts, list), f"guess_all type = {type(exts)!r}"
assert len(exts) >= 1, f"text/html has at least one ext: {exts!r}"
assert ".html" in exts, f".html in {exts!r}"
assert ".htm" in exts, f".htm in {exts!r}"
print("guess_all_extensions_list OK")
