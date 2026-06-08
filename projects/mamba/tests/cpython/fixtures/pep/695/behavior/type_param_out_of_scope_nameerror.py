# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_param_out_of_scope_nameerror"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "mamba raises a compile-time 'undefined name' for the class type param rather than the runtime NameError (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a class type param is visible only inside that class scope; using Tp after class Generic1[Tp] raises NameError mentioning 'Tp'"""


# A class type param is visible only inside that class' own scope.
try:
    class Generic1[Tp]:
        ...
    leaked = Tp  # noqa: F821 -- intentional out-of-scope use
    raise AssertionError("expected NameError for out-of-scope type param")
except NameError as exc:
    assert "Tp" in str(exc)

print("type_param_out_of_scope_nameerror OK")
