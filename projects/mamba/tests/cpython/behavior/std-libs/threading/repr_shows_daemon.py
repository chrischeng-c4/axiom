# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "repr_shows_daemon"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: repr() of a fresh thread omits 'daemon'; after setting daemon=True repr() includes 'daemon'"""
import threading

fresh = threading.Thread()
assert "daemon" not in repr(fresh), f"fresh repr = {repr(fresh)!r}"
fresh.daemon = True
assert "daemon" in repr(fresh), f"daemon repr = {repr(fresh)!r}"

print("repr_shows_daemon OK")
