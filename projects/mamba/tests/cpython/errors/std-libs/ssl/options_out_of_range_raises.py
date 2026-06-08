# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "options_out_of_range_raises"
# subject = "ssl.SSLContext.options"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.options: the options bitmask rejects bad values: -1 and 2**100 raise OverflowError, a str raises TypeError"""
import ssl

_opt = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
assert isinstance(_opt.options, int), "options is int"
for _bad, _exc in ((-1, OverflowError), (2 ** 100, OverflowError), ("abc", TypeError)):
    try:
        _opt.options = _bad
        raise AssertionError(f"options={_bad!r} should raise")
    except (OverflowError, TypeError) as _e:
        assert isinstance(_e, _exc), f"options={_bad!r} -> {type(_e).__name__}"

print("options_out_of_range_raises OK")
