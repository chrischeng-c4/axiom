# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "main_thread_named_mainthread"
# subject = "threading.main_thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.main_thread: main_thread() is named 'MainThread' and its ident equals current_thread().ident and get_ident() on the main thread"""
import threading

main = threading.main_thread()
assert main.name == "MainThread", f"main name = {main.name!r}"
assert main.ident == threading.current_thread().ident, "main ident == current ident"
assert main.ident == threading.get_ident(), "main ident == get_ident()"

print("main_thread_named_mainthread OK")
