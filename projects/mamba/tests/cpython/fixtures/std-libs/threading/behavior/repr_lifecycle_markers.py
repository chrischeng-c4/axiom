# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "repr_lifecycle_markers"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: repr() shows the lifecycle markers 'initial' before start, 'started' while running, and 'stopped' after join"""
import threading

gate = threading.Event()

def hold():
    gate.wait()

live = threading.Thread(target=hold)
assert "initial" in repr(live), f"initial repr = {repr(live)!r}"
live.start()
assert "started" in repr(live), f"started repr = {repr(live)!r}"
gate.set()
live.join()
assert "stopped" in repr(live), f"stopped repr = {repr(live)!r}"

print("repr_lifecycle_markers OK")
