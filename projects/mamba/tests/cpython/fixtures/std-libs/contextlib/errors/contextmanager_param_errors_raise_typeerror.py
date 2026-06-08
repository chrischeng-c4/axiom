# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_param_errors_raise_typeerror"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager-wrapped factory validates its own signature: calling it with missing/extra/wrong arguments raises TypeError before any with-block"""
import contextlib


@contextlib.contextmanager
def needs_args(a, *, b):
    yield (a, b)


# Each bad call mis-supplies the signature: no args, too many positionals,
# and missing the required positional. All must raise TypeError at call time,
# before any with-block is entered.
for bad in (lambda: needs_args(), lambda: needs_args(1, 2), lambda: needs_args(b=3)):
    _raised = False
    try:
        bad()
    except TypeError:
        _raised = True
    assert _raised, "expected TypeError for malformed contextmanager call"

# The well-formed call still works.
with needs_args(1, b=2) as pair:
    assert pair == (1, 2), pair

print("contextmanager_param_errors_raise_typeerror OK")
