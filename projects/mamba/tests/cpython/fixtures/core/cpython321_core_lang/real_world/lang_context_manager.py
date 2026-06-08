# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_context_manager"
# subject = "cpython321.lang_context_manager"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_context_manager.py"
# status = "filled"
# ///
"""cpython321.lang_context_manager: execute CPython 3.12 seed lang_context_manager"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the class-based context-manager
# protocol (`__enter__` / `__exit__`).
# Surface: `with cm as v` calls __enter__ and binds its return value
# to v; on normal block exit __exit__ runs; on exceptional exit
# __exit__ still runs (cleanup contract). The exception-suppression
# behaviour (truthy __exit__ swallows the exception) is NOT asserted
# here — mamba currently propagates the exception through the `with`
# block regardless of the __exit__ return value.
_ledger: list[int] = []

class CM:
    def __init__(self, n):
        self.n = n
        self.log: list[str] = []
    def __enter__(self):
        self.log.append("enter")
        return self.n
    def __exit__(self, typ, val, tb):
        self.log.append("exit")
        return None

# Normal-path: enter, body, exit (in order)
cm = CM(42)
with cm as v:
    cm.log.append(f"body:{v}")
assert cm.log == ["enter", "body:42", "exit"]; _ledger.append(1)
# The value bound by `as v` is what __enter__ returned
assert v == 42; _ledger.append(1)

# Re-entering a fresh CM: entered + exited in order
cm2 = CM(99)
with cm2 as v2:
    pass
assert cm2.log == ["enter", "exit"]; _ledger.append(1)
assert v2 == 99; _ledger.append(1)

# Returning a falsy value from __exit__ propagates the exception, and
# __exit__ still runs (cleanup invariant)
class CMNoSuppress:
    def __init__(self):
        self.log: list[str] = []
    def __enter__(self):
        self.log.append("enter")
        return self
    def __exit__(self, typ, val, tb):
        self.log.append("exit")
        return False

cm3 = CMNoSuppress()
caught = ""
try:
    with cm3:
        raise ValueError("oops")
except ValueError as e:
    caught = str(e)
assert caught == "oops"; _ledger.append(1)
# __exit__ still ran on the exception path
assert cm3.log == ["enter", "exit"]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_context_manager {sum(_ledger)} asserts")
