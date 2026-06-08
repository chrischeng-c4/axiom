# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "auto_thread_name"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: an unnamed Thread gets an auto-generated 'Thread-N' name"""
import threading

auto = threading.Thread()
assert auto.name.startswith("Thread-"), f"auto name = {auto.name!r}"

print("auto_thread_name OK")
