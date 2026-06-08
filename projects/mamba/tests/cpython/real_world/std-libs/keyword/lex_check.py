# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "real_world"
# case = "lex_check"
# subject = "keyword.iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""keyword.iskeyword: static checker partitions a candidate name list into reserved-rejected and accepted, soft keywords accepted"""
import keyword

# Candidate variable names from a hypothetical config parse.
candidates = [
    "user_id",
    "class",       # hard keyword - must be rejected
    "match",       # soft keyword - allowed as identifier
    "retry_count",
    "if",          # hard keyword - must be rejected
    "total",
    "type",        # soft keyword - allowed as identifier
]

rejected = [name for name in candidates if keyword.iskeyword(name)]
accepted = [name for name in candidates if not keyword.iskeyword(name)]

assert rejected == ["class", "if"], f"unexpected rejected={rejected!r}"
assert accepted == ["user_id", "match", "retry_count", "total", "type"], (
    f"unexpected accepted={accepted!r}"
)

# Soft keywords must NOT be flagged hard, but must be flagged soft.
for soft in keyword.softkwlist:
    assert not keyword.iskeyword(soft), f"softkw {soft!r} wrongly flagged hard"
    assert keyword.issoftkeyword(soft), f"softkw {soft!r} not flagged soft"

print("lex_check OK")
