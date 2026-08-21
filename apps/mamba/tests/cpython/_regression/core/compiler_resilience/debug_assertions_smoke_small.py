# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Small crash-corpus smoke for the debug-assertions lane."""


def nested_expr(depth: int) -> str:
    return "(" * depth + "1" + ")" * depth


def nested_fstring(depth: int) -> str:
    inner = "1"
    for level in range(depth):
        quote = '"' if level % 2 == 0 else "'"
        inner = "f" + quote + "{" + inner + "}" + quote
    return "value = " + inner + "\n"


cases = [
    ("nested_expr", nested_expr(60), "eval"),
    ("nested_fstring", nested_fstring(12), "exec"),
    ("many_statements", "x=1\n" * 4000, "exec"),
]

results: list[str] = []
for label, source, mode in cases:
    code = compile(source, f"<{label}>", mode)
    ns: dict[str, object] = {}
    if mode == "exec":
        exec(code, ns)
        if label == "nested_fstring":
            assert ns["value"] == "1"
    else:
        assert eval(code) == 1
    results.append(label)

print("debug_assertions_smoke_cases:", ",".join(results))
print("debug_assertions_smoke_small: OK")
