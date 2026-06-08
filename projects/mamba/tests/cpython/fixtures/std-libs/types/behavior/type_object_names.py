# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "type_object_names"
# subject = "types.NoneType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.NoneType: the named type objects (FunctionType/MethodType/ModuleType/GeneratorType/CoroutineType/NoneType/MappingProxyType/EllipsisType/NotImplementedType) expose POSIX-stable __name__ values matching CPython"""
import types

# Type objects expose __name__ for introspection — POSIX-stable names that
# match CPython exactly.
expected = {
    "FunctionType": "function",
    "MethodType": "method",
    "ModuleType": "module",
    "GeneratorType": "generator",
    "CoroutineType": "coroutine",
    "NoneType": "NoneType",
    "MappingProxyType": "mappingproxy",
    "EllipsisType": "ellipsis",
    "NotImplementedType": "NotImplementedType",
}
for attr, name in expected.items():
    obj = getattr(types, attr)
    assert obj.__name__ == name, (attr, obj.__name__, name)

print("type_object_names OK")
