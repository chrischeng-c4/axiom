# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "excepthook_default_restorable"
# subject = "threading.excepthook"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.excepthook: threading.__excepthook__ is the preserved default and excepthook can be restored to it after temporary replacement"""
import threading

original = threading.excepthook
threading.excepthook = lambda args: None
threading.excepthook = original
assert threading.__excepthook__ is not None, "default excepthook present"
assert threading.excepthook is original, "excepthook restored"

print("excepthook_default_restorable OK")
