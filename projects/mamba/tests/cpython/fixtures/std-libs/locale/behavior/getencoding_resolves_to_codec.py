# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "getencoding_resolves_to_codec"
# subject = "locale.getencoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.getencoding: getencoding returns a non-empty codec-resolvable encoding name"""
import codecs
import locale

enc = locale.getencoding()
assert isinstance(enc, str), "getencoding -> str"
assert enc != "", "getencoding non-empty"
codecs.lookup(enc)  # raises LookupError if not a real codec

print("getencoding_resolves_to_codec OK")
