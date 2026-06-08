# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "daemon_flag_roundtrip"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: the Thread(daemon=...) constructor flag is observable via the .daemon attribute for both True and False"""
import threading

_td = threading.Thread(target=lambda: None, daemon=True)
assert _td.daemon, "daemon thread is daemon"
_tn = threading.Thread(target=lambda: None, daemon=False)
assert not _tn.daemon, "non-daemon thread not daemon"

print("daemon_flag_roundtrip OK")
