# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "behavior"
# case = "annotated_with_value_is_class_attr"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "class __annotations__ / annotation-only attribute machinery diverges on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: an annotated class attribute WITH a value (`y: str = 'hi'`) becomes a real class attribute, while a bare annotation (`x: int`) does NOT"""


class A:
    x: int  # bare annotation: documented, not bound
    y: str = "hi"  # annotation + value: a real class attribute


assert not hasattr(A, "x"), "bare annotation must NOT create a class attribute"
assert hasattr(A, "y"), "annotation with value IS a class attribute"
assert A.y == "hi", A.y
print("annotated_with_value_is_class_attr OK")
