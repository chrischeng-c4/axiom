# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""str.format: conversions, nested specs, numbering rules (CPython 3.12)."""

# --- conversion flags on str.format -----------------------------------
assert "{!r}".format("s") == "'s'"
assert "{!s}".format(42) == "42"
assert "{!a}".format(42) == "42"
# !a uses ascii(): non-ASCII becomes a \x / \u escape.
assert "{!a}".format("é") == "'\\xe9'"
print("conversions:", "{!r}/{!s}".format("a", "b"))

# --- nested format specs (spec drawn from an argument) ----------------
# The width / alignment inside {:...} can itself be a replacement field.
assert "{:{}}".format("bar", 6) == "bar   "
assert "{:^{}}".format("bar", 7) == "  bar  "
assert "a{:{}x}b".format(20, "#") == "a0x14b"     # type char after nested
assert "{:{f}}{g}{}".format(1, 3, g="g", f=" ") == " 1g3"
print("nested:", "{:>{}}".format("x", 5))

# --- a custom __format__ receives the resolved spec -------------------
class Spec:
    def __format__(self, spec):
        return "spec=" + spec

assert "{:^+10.3f}".format(Spec()) == "spec=^+10.3f"
assert "{}".format(Spec()) == "spec="
print("dunder:", "{:hello}".format(Spec()))

# --- auto-numbering may not be mixed with manual numbering ------------
assert "a{}b{}c".format(0, 1) == "a0b1c"
assert "{0}-{0}".format("x") == "x-x"
for bad in ("{}{1}", "{1}{}", "{0:{}}", "{:{1}}"):
    raised = False
    try:
        bad.format(1, 2)
    except ValueError:
        raised = True
    assert raised, bad
print("numbering rules enforced")

# --- a subclass __str__ is honored by {} but not by {!r} -------------
class S(str):
    def __str__(self):
        return "overridden"

assert "{}".format(S("xxx")) == "overridden"
assert "%s" % S("xxx") == "overridden"

print("format_conversions_nested OK")
