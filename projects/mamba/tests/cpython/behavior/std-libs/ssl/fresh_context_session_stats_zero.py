# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "fresh_context_session_stats_zero"
# subject = "ssl.SSLContext.session_stats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.session_stats: a fresh context's session_stats() is an all-zero dict with the documented 'number' and 'hits' keys"""
import ssl

_stats = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT).session_stats()
assert set(_stats.values()) == {0}, f"fresh stats all zero: {_stats!r}"
assert _stats["number"] == 0 and _stats["hits"] == 0, "stats keys present"

print("fresh_context_session_stats_zero OK")
