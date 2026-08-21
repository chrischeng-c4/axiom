# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/compiler_resilience: long chained comparisons + boolean ops (CPython 3.12 oracle)."""
# mamba-xfail: compiler has no parser bounds — hostile source crashes/diverges (WI #3929)
#
# Source is built at RUNTIME and fed to compile()/eval(). A long chained
# comparison (a < b < c < ...) and a long boolean-op chain (x and x and ...)
# both produce wide-but-shallow AST nodes that CPython folds + compiles cleanly.
# mamba has no operand-count bound on Compare/BoolOp and risks divergence on the
# same input (project_mamba_compiler_hostile_source_dos).

# Case 1: a long chained comparison over ascending integer literals.
# CPython desugars `a < b < c` into pairwise comparisons joined by short-circuit.
n = 2_000
chain = " < ".join(str(i) for i in range(n))     # "0 < 1 < 2 < ... < 1999"
src_cmp = "result = (" + chain + ")\n"
ns = {}
exec(compile(src_cmp, "<long-chain>", "exec"), ns)
assert ns["result"] is True                       # strictly ascending -> True
print("chain_operands:", n)
print("chain_result:", ns["result"])

# A chain that breaks monotonicity short-circuits to False, cleanly.
src_false = "result = (0 < 1 < 1 < 2)\n"           # 1 < 1 is False
ns_f = {}
exec(compile(src_false, "<chain-false>", "exec"), ns_f)
assert ns_f["result"] is False
print("chain_false_ok: True")

# Case 2: a long boolean-op chain. `True and True and ...` is one BoolOp node
# with many values; CPython handles an unbounded operand count cleanly.
m = 2_000
bool_src = "flag = (" + " and ".join("True" for _ in range(m)) + ")\n"
ns2 = {}
exec(compile(bool_src, "<long-and>", "exec"), ns2)
assert ns2["flag"] is True
print("bool_operands:", m)

# `or` chain that resolves to the first truthy operand.
or_src = "first = (" + " or ".join(["0"] * 100 + ["42"]) + ")\n"
ns3 = {}
exec(compile(or_src, "<long-or>", "exec"), ns3)
assert ns3["first"] == 42
print("or_chain_first:", ns3["first"])

mamba_note = (
    "mamba has no operand-count bound on Compare/BoolOp; long chains risk "
    "compiler divergence (project_mamba_compiler_hostile_source_dos)"
)
print("mamba_note:", mamba_note)

print("chained_comps: CPython compiled long chains cleanly, no crash — OK")
