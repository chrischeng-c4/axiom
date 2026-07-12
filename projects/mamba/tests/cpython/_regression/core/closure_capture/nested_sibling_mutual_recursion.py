# HANDWRITE-BEGIN gap="missing-generator:mamba-nested-sibling-recursion-tests" tracker="#1477" reason="A one-case CPython oracle fixture must prove both nested siblings resolve the same callable cell after all definitions execute."
def sibling_parity():
    def is_even(value):
        return True if value == 0 else is_odd(value - 1)

    def is_odd(value):
        return False if value == 0 else is_even(value - 1)

    return is_even(4), is_odd(5)


print("[closure] nested-sibling-mutual-recursion:", sibling_parity())

<!-- marker: missing-generator:mamba-nested-sibling-recursion-tests path: projects/mamba/tests/cpython/_regression/core/closure_capture/nested_sibling_mutual_recursion.py reason: A one-case CPython oracle fixture must prove both nested siblings resolve the same callable cell after all definitions execute. -->
# HANDWRITE-END
