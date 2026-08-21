# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "scanner_tokenizes"
# subject = "re.Scanner"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Scanner: re.Scanner walks the input calling per-pattern callbacks (a None callback consumes silently); scan() returns (tokens, remainder)"""
import re

def t_ident(scanner, token):
    return ("ID", token)


def t_int(scanner, token):
    return ("INT", int(token))


def t_op(scanner, token):
    return ("OP", token)


scanner = re.Scanner([
    (r"[a-zA-Z_]\w*", t_ident),
    (r"\d+", t_int),
    (r"[=+\-*/]", t_op),
    (r"\s+", None),            # whitespace: skipped, emits no token
])

tokens, remainder = scanner.scan("sum = 3 * foo + 42")
assert tokens == [
    ("ID", "sum"), ("OP", "="), ("INT", 3), ("OP", "*"),
    ("ID", "foo"), ("OP", "+"), ("INT", 42),
], f"tokens = {tokens!r}"
assert remainder == "", f"remainder = {remainder!r}"

# Unrecognized leading input is left in the remainder.
tokens2, remainder2 = scanner.scan("# ab")
assert tokens2 == [], f"tokens2 = {tokens2!r}"
assert remainder2 == "# ab", f"remainder2 = {remainder2!r}"

print("scanner_tokenizes OK")
