# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/compiler_resilience: tens of thousands of statements (CPython 3.12 oracle)."""
# mamba-xfail: compiler has no parser bounds — hostile source crashes/diverges (WI #3929)
#
# Source is built at RUNTIME (50k assignments) and handed to compile(). CPython
# compiles a flat module body of 50k statements without issue — the body is a
# linear list, not recursive — so this is a clean, finite compile. mamba has no
# statement-count bound and risks AST bloat / divergence on the same input
# (project_mamba_compiler_hostile_source_dos).

stmt_count = 50_000
src = "x=1\n" * stmt_count
print("stmt_count:", stmt_count)
print("source_bytes:", len(src))

# CPython 3.12: compiles cleanly to a module-level code object.
code = compile(src, "<many-stmts>", "exec")
assert code is not None
ns = {}
exec(code, ns)
assert ns["x"] == 1
print("compiled_and_executed: True")

# Sanity: a distinct-name variant still compiles, so the limit (if any) is not
# about the number of distinct locals being tiny — it is the statement count
# that is unbounded. Keep this variant smaller to stay fast and deterministic.
distinct = 5_000
src2 = "".join(f"v{i}=1\n" for i in range(distinct))
code2 = compile(src2, "<distinct-names>", "exec")
ns2 = {}
exec(code2, ns2)
assert ns2["v0"] == 1 and ns2[f"v{distinct - 1}"] == 1
print("distinct_names:", distinct)

# Document the mamba risk explicitly.
mamba_note = (
    "mamba has no statement-count cap; 50k-statement module risks AST bloat / "
    "compiler divergence (project_mamba_compiler_hostile_source_dos)"
)
print("mamba_note:", mamba_note)

print("many_stmts: CPython compiled 50k statements, no crash — OK")
