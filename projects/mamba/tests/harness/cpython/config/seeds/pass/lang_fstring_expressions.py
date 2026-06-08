# Operational AssertionPass seed for f-string expression embedding
# and format-spec surfaces beyond lang_pep701_fstring.
# Surface: arithmetic, function call, indexing, comparison inside
# `{...}`; integer base format specs (b/o/x); zero-pad with width;
# center alignment `^`; literal-brace escape via `{{` / `}}`.
_ledger: list[int] = []

# Arithmetic expression inside the placeholder
assert f"sum:{1 + 2 + 3}" == "sum:6"; _ledger.append(1)

# Function call inside the placeholder
assert f"len:{len('hello')}" == "len:5"; _ledger.append(1)

# Indexing inside the placeholder
nums = [10, 20, 30]
assert f"first:{nums[0]}" == "first:10"; _ledger.append(1)

# Comparison inside the placeholder — boolean repr
x = 10
assert f"compare:{x > 5}" == "compare:True"; _ledger.append(1)

# Integer base format specs
assert f"{255:x}" == "ff"; _ledger.append(1)
assert f"{255:X}" == "FF"; _ledger.append(1)
assert f"{8:o}" == "10"; _ledger.append(1)
assert f"{5:b}" == "101"; _ledger.append(1)

# Zero-pad with width
assert f"{42:05d}" == "00042"; _ledger.append(1)
assert f"{42:08d}" == "00000042"; _ledger.append(1)

# Center alignment with explicit fill character
assert f"{'x':*^7}" == "***x***"; _ledger.append(1)
# Left and right alignment with explicit fill
assert f"{'x':-<5}" == "x----"; _ledger.append(1)
assert f"{'x':->5}" == "----x"; _ledger.append(1)

# Literal-brace escape — `{{` and `}}` become single braces
assert f"escape:{{X}}" == "escape:{X}"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_fstring_expressions {sum(_ledger)} asserts")
