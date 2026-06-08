# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "pthread_getcpuclockid__thread_id_as_int_wrong"
# subject = "time.pthread_getcpuclockid(thread_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.pthread_getcpuclockid(thread_id: int); call it with the wrong type.

typeshed contract: thread_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import pthread_getcpuclockid
try:
    pthread_getcpuclockid("not_an_int")  # thread_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
