# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_returns_anchored_regex"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.translate: translate returns a str regex wrapped in (?s:...)\\Z that is anchored to the full string: the compiled regex matches 'script.py' but not 'script.py.bak'"""
import re
import fnmatch

_re_str = fnmatch.translate("*.py")
assert isinstance(_re_str, str), f"translate type = {type(_re_str)!r}"
_pat = re.compile(_re_str)
assert _pat.match("script.py"), "anchored regex matches full string"
assert not _pat.match("script.py.bak"), "anchored at end rejects trailing text"

print("translate_returns_anchored_regex OK")
