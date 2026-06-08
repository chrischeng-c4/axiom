# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Local / global / nonlocal scope resolution — #2780.
#
# Covers Python's name-resolution rules across three lexical scopes:
#
#   local       Names assigned (without `global`/`nonlocal`) inside a
#               function are local to that function. Reading a name
#               that's later assigned in the same function but BEFORE
#               the assignment raises UnboundLocalError.
#   global      `global x` inside a function declares x lives at
#               module scope; assignment writes through.
#   nonlocal    `nonlocal x` declares x lives in the nearest enclosing
#               function scope (NOT module scope, NOT inside a class).
#
# Clauses:
#   1. Plain local assignment shadows the enclosing name.
#   2. `global` write-through mutates the module-level binding.
#   3. `nonlocal` write-through mutates the enclosing function's
#      binding (but not the module's same-named binding).
#   4. Assignment without `global`/`nonlocal` creates a NEW local
#      that shadows but doesn't mutate the outer name.
#   5. UnboundLocalError when a name is read before its local
#      assignment.
#   6. `nonlocal` cannot reach module scope — declaring `nonlocal`
#      on a name that doesn't exist in any enclosing function scope
#      is a SyntaxError at compile time (asserted via exec).
#
# Every print line tagged `[scope]` so failure output names scope
# semantics.

state = "module"
counter = 0


# Clause 1: plain local assignment shadows the module binding inside
# the function; module-level `state` is unchanged on return.
def clause1_local():
    state = "local"
    return state


print("[scope] clause-1 inner-return:", clause1_local())
print("[scope] clause-1 module-after:", state)


# Clause 2: `global` write-through.
def clause2_global_write():
    global counter
    counter += 10
    return counter


print("[scope] clause-2 inner-after-write:", clause2_global_write())
print("[scope] clause-2 module-counter-after:", counter)


# Clause 3: `nonlocal` write-through. `state` here is the
# OUTER-FUNCTION local, NOT the module-level `state` (Python's
# nonlocal cannot reach module scope).
def clause3_outer():
    state = "outer"

    def inner():
        nonlocal state
        state = "inner-wrote"

    inner()
    return state


print("[scope] clause-3 outer-return:", clause3_outer())
# Module `state` is still "module" — nonlocal did NOT promote up to
# module scope.
print("[scope] clause-3 module-state:", state)


# Clause 4: plain assignment in the inner function creates a NEW
# local that shadows the outer name. Outer name unchanged on return.
def clause4_outer():
    state = "outer"

    def inner():
        state = "inner-local"
        return state

    inner_value = inner()
    return state, inner_value


outer_state, inner_state = clause4_outer()
print("[scope] clause-4 outer-state:", outer_state)
print("[scope] clause-4 inner-state:", inner_state)


# Clause 5: UnboundLocalError — `state` is assigned somewhere in this
# function body, so it is a local; reading it BEFORE the assignment
# is an error.
def clause5_unbound():
    try:
        _ = state  # pyright: ignore[reportUnboundVariable] — intentional test of UnboundLocalError
        return "no-error"
    except UnboundLocalError as exc:
        return type(exc).__name__
    finally:
        # Assignment after the read; presence is what makes `state`
        # local in this scope.
        state = "assigned"
        # Reference to silence Pyright "unused" warnings.
        _ = state


print("[scope] clause-5 unbound-local-error:", clause5_unbound())


# Clause 6: `nonlocal` declared for a name that doesn't exist in any
# enclosing function scope is a compile-time SyntaxError. Use exec
# so the SyntaxError is raised at compile time of the snippet, not
# this module.
src = """
def outer():
    def inner():
        nonlocal missing
        missing = 1
    return inner
"""

try:
    compile(src, "<clause-6>", "exec")
    print("[scope] clause-6 syntaxerror: <unexpected-no-error>")
except SyntaxError as exc:
    print("[scope] clause-6 syntaxerror:", type(exc).__name__)
