# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "real_world"
# case = "lint_reserved_names"
# subject = "keyword.iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""keyword.iskeyword: code-generator rejects candidate names colliding with hard keywords while allowing soft keywords as identifiers"""
import keyword


def safe_identifier(candidate: str) -> bool:
    # A code-generator must not emit a name that is a Python hard keyword.
    return not keyword.iskeyword(candidate)


# Hard keywords must be rejected.
for word in ["class", "def", "return", "async", "await"]:
    assert not safe_identifier(word), f"{word!r} should be rejected"

# Ordinary identifiers must be accepted.
for word in ["user_id", "HTTPResponse", "_internal"]:
    assert safe_identifier(word), f"{word!r} should be accepted"

# Soft keywords are not reserved at the language level: iskeyword is False, so
# they are accepted; issoftkeyword covers that axis instead.
for word in ["match", "case", "type"]:
    assert safe_identifier(word), f"soft keyword {word!r} should be accepted"
    assert keyword.issoftkeyword(word), f"{word!r} should be a soft keyword"

# Every kwlist entry round-trips through iskeyword, including async/await.
for word in keyword.kwlist:
    assert keyword.iskeyword(word), f"kwlist member {word!r} not flagged"
assert "async" in keyword.kwlist
assert "await" in keyword.kwlist

print("lint_reserved_names OK")
