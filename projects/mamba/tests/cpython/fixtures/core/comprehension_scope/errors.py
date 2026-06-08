# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/comprehension_scope: scope / syntax error paths (CPython 3.12 oracle)."""


# Access undefined name raises NameError.
try:
    no_such_name_xyzzy  # type: ignore[name-defined]  # noqa: F821
    print("undef: no_raise")
except NameError as e:
    print("undef:", type(e).__name__, str(e)[:60])


# A comprehension runs in its own scope: the loop variable does not leak,
# so reading it afterwards in a function raises a NameError. (Under CPython
# 3.12 the comprehension still has its own frame, so the trailing `x` is a
# free/global lookup; UnboundLocalError is a NameError subclass, so catching
# NameError works regardless of which the interpreter chooses.)
def leak_after():
    [x for x in [1, 2, 3]]  # noqa: B007
    return x  # x was only ever bound inside the comprehension's scope


try:
    leak_after()
    print("after: no_raise")
except NameError as e:
    print("after:", type(e).__name__)


# Referencing a target before it is bound INSIDE the comprehension also
# raises: `l[0]` reads `l` before the for-target rebinds it.
def unbound_inside():
    l = [None]
    return [1 for l[0], l in [[1, 2]]]


try:
    unbound_inside()
    print("inside: no_raise")
except NameError as e:
    print("inside:", type(e).__name__)


# A comprehension cannot be used as an assignment target (SyntaxError).
try:
    compile("[y for y in (1, 2)] = 10", "<t>", "exec")
    print("assign: no_raise")
except SyntaxError as e:
    print("assign:", type(e).__name__, "cannot assign" in str(e))


# Nor as an augmented-assignment target.
try:
    compile("[y for y in (1, 2)] += 10", "<t>", "exec")
    print("augassign: no_raise")
except SyntaxError as e:
    print("augassign:", type(e).__name__, "illegal expression" in str(e))
