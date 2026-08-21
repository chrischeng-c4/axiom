# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "enumerate_tracks_live_threads"
# subject = "threading.enumerate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.enumerate: a running thread is a member of enumerate(); after it is joined it is no longer enumerated"""
import threading

gate = threading.Event()

def hold():
    gate.wait()

held = threading.Thread(target=hold)
held.start()
assert held in threading.enumerate(), "running thread is enumerated"
gate.set()
held.join()
assert held not in threading.enumerate(), "joined thread is not enumerated"

print("enumerate_tracks_live_threads OK")
