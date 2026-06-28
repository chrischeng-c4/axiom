"""compile() CodeObject inputs are executable by eval() and exec()."""

assert eval(compile("40 + 2", "<eval-code>", "eval")) == 42

namespace = {}
exec(compile("x = 5\ny = x * 3", "<exec-code>", "exec"), namespace)
assert namespace["x"] == 5
assert namespace["y"] == 15

single_namespace = {}
exec(compile("z = 9", "<single-code>", "single"), single_namespace)
assert single_namespace["z"] == 9

print("eval_exec_compile_codeobject OK")
