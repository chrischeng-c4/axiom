# Default argument evaluation — #2776.
#
# Covers Python's "defaults are evaluated once at function-definition time,
# not per call" semantics, including the gotcha that a mutable default
# persists across calls.
#
# Clauses:
#   1. Immutable defaults — definition-time evaluation visible via a
#      counter expression. `len(_marks)` is captured at definition, so
#      later mutations to `_marks` do NOT change the default.
#   2. Mutable default persistence — appending to a list default mutates
#      the function's shared default object, so subsequent calls see
#      prior mutations.
#   3. Per-call isolation when the user supplies their own arg — passing
#      `xs=[...]` bypasses the shared default; the shared default still
#      retains state from earlier no-arg calls.
#   4. id() of the default — the same list object is reused across no-arg
#      calls; passing a fresh list yields a different id.
#   5. Default expression evaluation order — defaults evaluate
#      left-to-right at def-time, captured exactly once.
#
# The fixture fails (CPython reference mismatch) if defaults are
# evaluated per call (clause 1 would diverge, clause 2 would always
# print []) or if mutable defaults are silently copied (clause 4 ids
# would differ between no-arg calls). Every print line is tagged with
# `[default-args]` so failure output names default-argument semantics.

# Clause 1: immutable default captured at definition time.
# `_marks` will mutate after the def; the default value embedded in
# `snapshot` is the LIST's length at def-time, which is 0.
_marks = []
def snapshot(n=len(_marks)):
    return n

_marks.append("post-def")
_marks.append("more")
print("[default-args] clause-1 captured-at-def-time:", snapshot())
print("[default-args] clause-1 _marks-now:", len(_marks))


# Clause 2: mutable default persists across calls.
# Each call without `xs=` mutates the same default list. Required by
# Python's data model — defaults are stored on the function object.
def collect(item, xs=[]):
    xs.append(item)
    return xs

a = collect("a")
b = collect("b")
c = collect("c")
print("[default-args] clause-2 first-call:", a)
print("[default-args] clause-2 second-call:", b)
print("[default-args] clause-2 third-call:", c)
print("[default-args] clause-2 same-object:", a is b is c)


# Clause 3: user-supplied arg bypasses the shared default.
# After clause 2 the shared default is ['a','b','c']. A call with an
# explicit `xs=` argument does NOT mutate that shared list.
fresh = collect("z", xs=[])
print("[default-args] clause-3 fresh-call:", fresh)
# Re-call without args to inspect the still-mutated shared default.
print("[default-args] clause-3 shared-still:", collect("post-fresh"))


# Clause 4: id() of the shared default is stable across no-arg calls.
def grab(xs=[]):
    return id(xs)

id1 = grab()
id2 = grab()
print("[default-args] clause-4 id-stable-across-noarg:", id1 == id2)
user = []
print("[default-args] clause-4 id-differs-with-user-arg:", grab(user) != id1)


# Clause 5: default expressions evaluate left-to-right at def-time,
# captured exactly once. We use a counter side-effect to prove the
# `def` line evaluates each default expression once.
_count = [0]
def _next():
    _count[0] += 1
    return _count[0]

def trio(a=_next(), b=_next(), c=_next()):
    return (a, b, c)

# At the def line above, _next() ran three times: a=1, b=2, c=3.
# Subsequent calls reuse those captured values; they do NOT re-evaluate
# the default expression.
print("[default-args] clause-5 first-call:", trio())
print("[default-args] clause-5 second-call:", trio())
print("[default-args] clause-5 counter-after-def:", _count[0])
# An explicit arg overrides without re-evaluating the default expr.
print("[default-args] clause-5 override:", trio(a=100))
print("[default-args] clause-5 counter-after-override:", _count[0])
