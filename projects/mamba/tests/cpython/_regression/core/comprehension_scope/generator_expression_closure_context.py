# HANDWRITE-BEGIN gap="missing-generator:mamba-generator-expression-closure-oracle" tracker="#1490" reason="A one-case CPython oracle fixture must pin closure late binding through generator expression iteration after a prior list-comprehension closure."
"""Generator-expression closures retain caller context after a list-comprehension closure."""

late = [lambda: i for i in range(5)]
assert [fn() for fn in late] == [4, 4, 4, 4, 4]

gen_funcs = list(lambda: j for j in range(3))
assert [fn() for fn in gen_funcs] == [2, 2, 2]

print("[generator-expression-closure-context]", [fn() for fn in gen_funcs])

<!-- marker: missing-generator:mamba-generator-expression-closure-oracle path: projects/mamba/tests/cpython/_regression/core/comprehension_scope/generator_expression_closure_context.py reason: A one-case CPython oracle fixture must pin closure late binding through generator expression iteration after a prior list-comprehension closure. -->
# HANDWRITE-END
