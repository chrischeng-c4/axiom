# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "getpreferredencoding_resolves_to_codec"
# subject = "locale.getpreferredencoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.getpreferredencoding: getpreferredencoding returns a str; if non-empty it resolves via codecs.lookup"""
import codecs
import locale

pref = locale.getpreferredencoding()
assert isinstance(pref, str), "getpreferredencoding -> str"
if pref:
    codecs.lookup(pref)

print("getpreferredencoding_resolves_to_codec OK")
