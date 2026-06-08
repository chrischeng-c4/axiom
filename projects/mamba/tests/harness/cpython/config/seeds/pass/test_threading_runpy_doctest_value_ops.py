# Atomic 305 pass conformance — threading module (hasattr Thread/Lock/
# RLock/Condition/Semaphore/BoundedSemaphore/Event/Timer/Barrier/local/
# current_thread/active_count/enumerate/main_thread/get_ident +
# active_count returns 1 in main + get_ident returns int) + runpy
# module (hasattr run_module/run_path) + pkgutil module (hasattr iter_
# modules/walk_packages/get_data/ModuleInfo/resolve_name) + doctest
# module (hasattr testmod/testfile/DocTestSuite/DocFileSuite/run_doc
# string_examples/Example/DocTest/DocTestParser/DocTestRunner/Debug
# Runner/OutputChecker/ELLIPSIS/IGNORE_EXCEPTION_DETAIL/SKIP).
# All asserts match between CPython 3.12 and mamba.
import threading
import runpy
import pkgutil
import doctest


_ledger: list[int] = []

# 1) threading — hasattr core surface
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "BoundedSemaphore") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Timer") == True; _ledger.append(1)
assert hasattr(threading, "Barrier") == True; _ledger.append(1)
assert hasattr(threading, "local") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)
assert hasattr(threading, "enumerate") == True; _ledger.append(1)
assert hasattr(threading, "main_thread") == True; _ledger.append(1)
assert hasattr(threading, "get_ident") == True; _ledger.append(1)

# 2) threading — value contracts (conformant subset)
assert threading.active_count() == 1; _ledger.append(1)
assert isinstance(threading.get_ident(), int) == True; _ledger.append(1)

# 3) runpy — hasattr core surface
assert hasattr(runpy, "run_module") == True; _ledger.append(1)
assert hasattr(runpy, "run_path") == True; _ledger.append(1)

# 4) pkgutil — hasattr core surface (conformant subset)
assert hasattr(pkgutil, "iter_modules") == True; _ledger.append(1)
assert hasattr(pkgutil, "walk_packages") == True; _ledger.append(1)
assert hasattr(pkgutil, "get_data") == True; _ledger.append(1)
assert hasattr(pkgutil, "ModuleInfo") == True; _ledger.append(1)
assert hasattr(pkgutil, "resolve_name") == True; _ledger.append(1)

# 5) doctest — hasattr core surface
assert hasattr(doctest, "testmod") == True; _ledger.append(1)
assert hasattr(doctest, "testfile") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestSuite") == True; _ledger.append(1)
assert hasattr(doctest, "DocFileSuite") == True; _ledger.append(1)
assert hasattr(doctest, "run_docstring_examples") == True; _ledger.append(1)
assert hasattr(doctest, "Example") == True; _ledger.append(1)
assert hasattr(doctest, "DocTest") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestParser") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestRunner") == True; _ledger.append(1)
assert hasattr(doctest, "DebugRunner") == True; _ledger.append(1)
assert hasattr(doctest, "OutputChecker") == True; _ledger.append(1)
assert hasattr(doctest, "ELLIPSIS") == True; _ledger.append(1)
assert hasattr(doctest, "IGNORE_EXCEPTION_DETAIL") == True; _ledger.append(1)
assert hasattr(doctest, "SKIP") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_threading_runpy_doctest_value_ops {sum(_ledger)} asserts")
