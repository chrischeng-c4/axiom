# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Comprehension scope — #2800.
#
# Covers Python 3 comprehension scoping rules:
#
#   - Each of list / dict / set / generator comprehensions runs in
#     its OWN implicit function scope. The loop variable does NOT
#     leak into the enclosing scope.
#   - Comprehensions can READ outer-scope names (closures).
#   - Comprehensions iteration target shadows an outer name without
#     overwriting it.
#
# Clauses:
#   1. List comprehension does not leak its loop variable.
#   2. Dict comprehension does not leak its loop variable.
#   3. Set comprehension does not leak its loop variable.
#   4. Generator expression does not leak its loop variable (and the
#      generator itself is lazy).
#   5. Comprehension reads outer-scope names (closure capture).
#   6. Comprehension iteration target shadows an outer name without
#      overwriting it.
#
# Every print line tagged `[compr-scope]` so failure output names
# comprehension scope semantics.


def is_defined(name):
    """Probe whether `name` is bound in the caller's globals via
    eval. Returns True/False instead of letting pyright statically
    analyze direct references."""
    try:
        eval(name, globals())
        return True
    except NameError:
        return False


# Clause 1: list comprehension.
squares = [x * x for x in range(4)]
print("[compr-scope] clause-1 squares:", squares)
print("[compr-scope] clause-1 leaked:", is_defined("x"))


# Clause 2: dict comprehension.
mapping = {k: k * 10 for k in range(3)}
print("[compr-scope] clause-2 mapping:", mapping)
print("[compr-scope] clause-2 leaked:", is_defined("k"))


# Clause 3: set comprehension.
evens = {s for s in range(8) if s % 2 == 0}
print("[compr-scope] clause-3 evens:", sorted(evens))
print("[compr-scope] clause-3 leaked:", is_defined("s"))


# Clause 4: generator expression.
gen = (g * g for g in range(4))
# Generator is lazy — iterating realizes the values.
print("[compr-scope] clause-4 gen-list:", list(gen))
print("[compr-scope] clause-4 leaked:", is_defined("g"))


# Clause 5: closure capture — comprehension reads outer-scope names.
outer_scale = 100
scaled = [v * outer_scale for v in range(3)]
print("[compr-scope] clause-5 scaled:", scaled)


# Clause 6: comprehension target shadows but does not overwrite outer.
shadow = "OUTER"
result = [shadow for shadow in ["A", "B", "C"]]  # noqa: B023
print("[compr-scope] clause-6 result:", result)
print("[compr-scope] clause-6 outer:", shadow)
