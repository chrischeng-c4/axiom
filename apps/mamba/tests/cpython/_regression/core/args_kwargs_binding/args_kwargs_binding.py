# args/kwargs argument binding — #2777.
#
# Covers Python's function-argument binding rules for the
# positional / keyword / *args / **kwargs cocktail, plus the
# TypeError cases that must surface when the call shape is illegal.
#
# Clauses:
#   1. Positional binding into *args — extra positionals go into the
#      tuple, fixed positionals get their slot first.
#   2. Keyword binding into **kwargs — extra keywords go into the
#      dict, declared keyword params get filled first.
#   3. Order of evaluation — fixed positionals before *args, declared
#      kwargs before **kwargs.
#   4. Iterable unpack and mapping unpack feed the same machinery.
#   5. Duplicate-argument error — same arg passed both positionally
#      and via kwargs raises TypeError.
#   6. Unexpected keyword argument — a kwarg name not declared and not
#      absorbed by **kwargs raises TypeError.
#   7. Missing required argument — a required positional left
#      unfilled raises TypeError.
#
# Every print line is tagged with `[args-kwargs-binding]` so failure
# output names the language feature. Error-class names are stable
# across CPython 3.12 / mamba so we assert them; the human-readable
# error message text is NOT asserted (only stable-where-stable).

# Clause 1: positional + *args.
def pos_then_star(a, b, *rest):
    return (a, b, rest)

print("[args-kwargs-binding] clause-1 noextra:", pos_then_star(1, 2))
print("[args-kwargs-binding] clause-1 oneextra:", pos_then_star(1, 2, 3))
print("[args-kwargs-binding] clause-1 manyextra:", pos_then_star(1, 2, 3, 4, 5))


# Clause 2: declared kwarg + **kwargs.
def kw_then_dstar(a, b=10, **rest):
    return (a, b, sorted(rest.items()))

print("[args-kwargs-binding] clause-2 nokw:", kw_then_dstar(1))
print("[args-kwargs-binding] clause-2 declkw:", kw_then_dstar(1, b=200))
print("[args-kwargs-binding] clause-2 extras:", kw_then_dstar(1, b=200, c=3, d=4))
print("[args-kwargs-binding] clause-2 onlyextras:", kw_then_dstar(1, c=3, d=4))


# Clause 3: all four families together.
def full(a, b=2, *args, **kwargs):
    return (a, b, args, sorted(kwargs.items()))

print("[args-kwargs-binding] clause-3 minimal:", full(1))
print("[args-kwargs-binding] clause-3 starabsorb:", full(1, 20, 30, 40))
print("[args-kwargs-binding] clause-3 mixed:", full(1, 20, 30, k=99, j=88))


# Clause 4: iterable + mapping unpack.
print("[args-kwargs-binding] clause-4 listunpack:", full(*[1, 2, 3, 4]))
print("[args-kwargs-binding] clause-4 dictunpack:", full(**{"a": 1, "b": 2, "k": 99}))
print(
    "[args-kwargs-binding] clause-4 both:",
    full(*[1, 2, 3], **{"k": 99, "j": 88}),
)


# Clause 5: duplicate-argument error — `a` is bound both positionally
# (via *args expansion's first element) and via kwargs.
try:
    full(1, **{"a": 99})
except TypeError as exc:
    print("[args-kwargs-binding] clause-5 typeerror:", type(exc).__name__)


# Clause 6: unexpected keyword argument — `z` is not declared and
# there is no **kwargs to absorb it.
def strict(a, b):
    return a + b

try:
    strict(1, b=2, z=99)  # pyright: ignore[reportCallIssue]
except TypeError as exc:
    print("[args-kwargs-binding] clause-6 typeerror:", type(exc).__name__)


# Clause 7: missing required positional.
def needs_both(a, b):
    return a, b

try:
    needs_both(1)  # pyright: ignore[reportCallIssue]
except TypeError as exc:
    print("[args-kwargs-binding] clause-7 typeerror:", type(exc).__name__)


# Sanity: *args and **kwargs can both be empty when no extras are
# supplied. Common mistake is returning None for empty *args.
def collect_extras(*args, **kwargs):
    return args, sorted(kwargs.items())

print("[args-kwargs-binding] clause-sanity empty:", collect_extras())
print("[args-kwargs-binding] clause-sanity onlypos:", collect_extras(1, 2))
print("[args-kwargs-binding] clause-sanity onlykw:", collect_extras(x=1, y=2))
