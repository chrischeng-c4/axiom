# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "pthread_sigmask_is_callable"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: pthread_sigmask_is_callable (surface)."""
import signal

assert callable(signal.pthread_sigmask)
print("pthread_sigmask_is_callable OK")
