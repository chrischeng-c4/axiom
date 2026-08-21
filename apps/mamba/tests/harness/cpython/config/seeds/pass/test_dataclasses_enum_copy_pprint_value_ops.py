# Atomic 269 pass conformance — dataclasses module (hasattr dataclass/
# field/fields/asdict/astuple) + enum module (hasattr Enum/IntEnum/
# Flag/IntFlag/StrEnum/auto/unique/EnumType) + copy module (hasattr
# copy/deepcopy/Error + copy.copy/copy.deepcopy on list/dict/nested-
# list value contracts) + pprint module (hasattr pprint/pformat).
# All asserts match between CPython 3.12 and mamba.
import dataclasses
import enum
import copy
import pprint


_ledger: list[int] = []

# 1) dataclasses — hasattr core decorators
assert hasattr(dataclasses, "dataclass") == True; _ledger.append(1)
assert hasattr(dataclasses, "field") == True; _ledger.append(1)
assert hasattr(dataclasses, "fields") == True; _ledger.append(1)
assert hasattr(dataclasses, "asdict") == True; _ledger.append(1)
assert hasattr(dataclasses, "astuple") == True; _ledger.append(1)

# 2) enum — hasattr core classes
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "StrEnum") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)
assert hasattr(enum, "unique") == True; _ledger.append(1)
assert hasattr(enum, "EnumType") == True; _ledger.append(1)

# 3) copy — hasattr surface
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)
assert hasattr(copy, "Error") == True; _ledger.append(1)

# 4) copy.copy of list returns equal list
assert copy.copy([1, 2, 3]) == [1, 2, 3]; _ledger.append(1)
assert copy.copy([]) == []; _ledger.append(1)

# 5) copy.copy of dict returns equal dict
assert copy.copy({"a": 1, "b": 2}) == {"a": 1, "b": 2}; _ledger.append(1)
assert copy.copy({}) == {}; _ledger.append(1)

# 6) copy.deepcopy of nested list returns equal nested list
assert copy.deepcopy([[1, 2], [3, 4]]) == [[1, 2], [3, 4]]; _ledger.append(1)
assert copy.deepcopy([1, 2, 3]) == [1, 2, 3]; _ledger.append(1)

# 7) pprint — hasattr core surface
assert hasattr(pprint, "pprint") == True; _ledger.append(1)
assert hasattr(pprint, "pformat") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_dataclasses_enum_copy_pprint_value_ops {sum(_ledger)} asserts")
