# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "bounded_semaphore_over_release_raises"
# subject = "threading.BoundedSemaphore"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.BoundedSemaphore: BoundedSemaphore(2): acquire then release returns to the bound; one more release past the bound raises ValueError"""
import threading

sem = threading.BoundedSemaphore(2)
sem.acquire()
sem.release()  # back at the bound of 2
_raised = False
try:
    sem.release()  # past the bound
except ValueError:
    _raised = True
assert _raised, "expected ValueError on over-release"

print("bounded_semaphore_over_release_raises OK")
