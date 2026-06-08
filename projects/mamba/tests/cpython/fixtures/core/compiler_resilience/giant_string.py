# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/compiler_resilience: giant string literals + implicit concat (CPython 3.12 oracle)."""
# mamba-xfail: compiler has no parser bounds — hostile source crashes/diverges (WI #3929)
#
# Source is built at RUNTIME and handed to compile(). A single huge string
# literal and a long run of implicit-concatenation fragments both stress the
# tokenizer/AST string-buffer with no inherent bound. CPython absorbs these
# (large but finite allocation); mamba's unbounded literal handling can OOM /
# diverge the COMPILER (project_mamba_compiler_hostile_source_dos).

# Case 1: one large string literal (100k chars of payload inside quotes).
payload_len = 100_000
src_big = "x = '" + ("a" * payload_len) + "'\n"
code = compile(src_big, "<big-literal>", "exec")
ns = {}
exec(code, ns)
assert len(ns["x"]) == payload_len
print("big_literal_len:", len(ns["x"]))

# Case 2: many implicit-concatenation fragments — Python joins adjacent
# string literals at compile time with NO documented cap on the count.
fragments = 20_000
src_concat = "y = (" + " ".join("'z'" for _ in range(fragments)) + ")\n"
code2 = compile(src_concat, "<implicit-concat>", "exec")
ns2 = {}
exec(code2, ns2)
assert ns2["y"] == "z" * fragments
assert len(ns2["y"]) == fragments
print("concat_fragments:", fragments)
print("concat_len:", len(ns2["y"]))

# Document the note: CPython folds unbounded implicit concat into one constant
# at compile time. There is no SyntaxError for "too many fragments"; the only
# limit is available memory. Either a clean compile OR a clean MemoryError is
# acceptable — the contract is "no crash".
unbounded_concat_note = (
    "CPython implicit-concat is unbounded at parse time; "
    "folded to a single str constant; only memory limits it"
)
print("unbounded_concat_note:", unbounded_concat_note)

print("giant_string: compiled cleanly, no crash — OK")
