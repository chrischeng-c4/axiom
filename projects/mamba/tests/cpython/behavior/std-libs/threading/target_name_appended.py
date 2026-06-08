# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "target_name_appended"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: Thread(target=worker) auto-name starts with 'Thread-' and ends with the target's name in parens, '(worker)'"""
import threading

def worker():
    pass

named_target = threading.Thread(target=worker)
assert named_target.name.startswith("Thread-"), named_target.name
assert named_target.name.endswith("(worker)"), f"target name = {named_target.name!r}"

print("target_name_appended OK")
