# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# dimension = "core"
# case = "module_loop_rebinds_global_with_def_present"
# subject = "module-scope for-loop rebinding a global while a function is defined"
# kind = "oracle"
# source = "issue #4242"
# status = "filled"
# ///
"""Module-scope for loop rebinds a global that is read after the loop, with
a `def` present earlier in the module.

python3.12 prints `3`: the post-loop `print(acc)` observes the loop's three
`acc += 1` increments. This is the tier1 oracle fixture pinning the defect
bisected at apps/mamba/src/lower/hir_to_mir.rs:7367 (issue #4242) -- with a
function defined anywhere in the module, `module_has_closures` is set, the
for-loop lowering's global-sync suppression is skipped, and the post-loop
global read observes a value the loop body's per-iteration stores do not
keep in sync, printing `0` instead of `3`.
"""


def f() -> None: pass


acc = 0
for _ in range(3): acc += 1
print(acc)
