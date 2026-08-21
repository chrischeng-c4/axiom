# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""string.Template: $-substitution, braces, safe mode (CPython 3.12 oracle)."""

from string import Template

# --- substitute -------------------------------------------------------
# $name and ${name} both expand; $$ is a literal dollar sign.
t = Template("$who likes to eat a bag of $what worth $$100")
assert t.substitute(who="tim", what="ham") == "tim likes to eat a bag of ham worth $100"

# Braced placeholders allow adjacency with surrounding text.
braced = Template("$who likes ${what} for ${meal}")
assert braced.substitute(who="tim", what="ham", meal="dinner") == "tim likes ham for dinner"

# Identifiers may contain underscores and digits (but not lead with a digit).
odd = Template("$_wh0_ likes ${_w_h_a_t_} for ${mea1}")
assert odd.substitute(_wh0_="tim", _w_h_a_t_="ham", mea1="dinner") == "tim likes ham for dinner"

# A mapping argument works the same as keyword arguments.
assert Template("$a/$b").substitute({"a": "1", "b": "2"}) == "1/2"

# Unicode values pass through verbatim.
assert Template("$x").substitute(x="tÿm") == "tÿm"
print("substitute:", braced.substitute(who="x", what="y", meal="z"))

# --- missing keys -----------------------------------------------------
# substitute raises KeyError when a placeholder has no value.
raised = False
try:
    t.substitute(who="tim")
except KeyError:
    raised = True
assert raised

# safe_substitute leaves unknown placeholders untouched.
assert braced.safe_substitute(who="tim") == "tim likes ${what} for ${meal}"
assert braced.safe_substitute() == "$who likes ${what} for ${meal}"

# A resolved key is substituted even when others are missing.
assert braced.safe_substitute(what="ham") == "$who likes ham for ${meal}"
print("safe_substitute:", braced.safe_substitute(who="tim", what="ham", meal="dinner"))

print("string_template OK")
