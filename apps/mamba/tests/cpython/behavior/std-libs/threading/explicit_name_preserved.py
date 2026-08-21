# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "explicit_name_preserved"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: an explicit name='my-thread' is preserved verbatim even with a target, and is settable after construction"""
import threading

def worker():
    pass

explicit = threading.Thread(target=worker, name="my-thread")
assert explicit.name == "my-thread", f"explicit name = {explicit.name!r}"
explicit.name = "renamed"
assert explicit.name == "renamed", f"renamed = {explicit.name!r}"

print("explicit_name_preserved OK")
