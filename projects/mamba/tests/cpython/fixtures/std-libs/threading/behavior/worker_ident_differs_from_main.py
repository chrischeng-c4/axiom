# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "worker_ident_differs_from_main"
# subject = "threading.get_ident"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.get_ident: a worker thread sees get_ident() == current_thread().ident, both distinct from the main thread's ident"""
import threading

main = threading.main_thread()
idents = {}

def record():
    idents["worker"] = threading.get_ident()
    idents["current"] = threading.current_thread().ident

w = threading.Thread(target=record)
w.start()
w.join()
assert idents["worker"] == idents["current"], "get_ident == current_thread().ident in worker"
assert idents["worker"] != main.ident, "worker ident differs from main"

print("worker_ident_differs_from_main OK")
