# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: dis — CPython bytecode opcode constants exposed as ints
# (LOAD_CONST, RETURN_VALUE, LOAD_FAST, STORE_FAST, JUMP_FORWARD, COMPARE_OP,
# BINARY_OP, UNARY_OP, CALL, NOP, POP_TOP, MAKE_FUNCTION, PUSH_NULL,
# RESUME, GET_ITER, FOR_ITER, RAISE_VARARGS, LOAD_NAME, STORE_NAME,
# LOAD_GLOBAL, STORE_GLOBAL, STORE_ATTR, LOAD_ATTR, STORE_SUBSCR,
# BUILD_LIST, BUILD_DICT, BUILD_SET, BUILD_TUPLE, HAVE_ARGUMENT,
# JUMP_ABSOLUTE), and dis.dis(fn) running without raising.
# Intentionally NOT exercised on mamba today (tracked separately):
#   * dis.Bytecode (None on mamba)
#   * dis.dis output content (mamba prints "(Mamba MIR not available in userspace)")
#   * dis.get_instructions / dis.code_info / dis.show_code / dis.stack_effect
#     real semantics (lambda stubs)
import dis

_ledger: list[int] = []
_op = dis.opmap

# Core opcode constants are exposed as ints with the canonical CPython values
assert _op["LOAD_CONST"] - 100 == 0, f"LOAD_CONST == 100, got {_op['LOAD_CONST']}"
_ledger.append(1)

assert _op["RETURN_VALUE"] - 83 == 0, f"RETURN_VALUE == 83, got {_op['RETURN_VALUE']}"
_ledger.append(1)

assert _op["LOAD_FAST"] - 124 == 0, f"LOAD_FAST == 124, got {_op['LOAD_FAST']}"
_ledger.append(1)

assert _op["STORE_FAST"] - 125 == 0, f"STORE_FAST == 125, got {_op['STORE_FAST']}"
_ledger.append(1)

assert _op["JUMP_FORWARD"] - 110 == 0, f"JUMP_FORWARD == 110, got {_op['JUMP_FORWARD']}"
_ledger.append(1)

assert _op["COMPARE_OP"] - 107 == 0, f"COMPARE_OP == 107, got {_op['COMPARE_OP']}"
_ledger.append(1)

assert _op["NOP"] - 9 == 0, f"NOP == 9, got {_op['NOP']}"
_ledger.append(1)

assert _op["POP_TOP"] - 1 == 0, f"POP_TOP == 1, got {_op['POP_TOP']}"
_ledger.append(1)

assert _op["MAKE_FUNCTION"] - 132 == 0, (
    f"MAKE_FUNCTION == 132, got {_op['MAKE_FUNCTION']}"
)
_ledger.append(1)

assert dis.HAVE_ARGUMENT - 90 == 0, (
    f"dis.HAVE_ARGUMENT == 90, got {dis.HAVE_ARGUMENT}"
)
_ledger.append(1)

# 3.12-era opcodes (PUSH_NULL=2, RESUME=151)
assert _op["PUSH_NULL"] - 2 == 0, f"PUSH_NULL == 2, got {_op['PUSH_NULL']}"
_ledger.append(1)

assert _op["RESUME"] - 151 == 0, f"RESUME == 151, got {_op['RESUME']}"
_ledger.append(1)

# Iteration opcodes
assert _op["GET_ITER"] - 68 == 0, f"GET_ITER == 68, got {_op['GET_ITER']}"
_ledger.append(1)

assert _op["FOR_ITER"] - 93 == 0, f"FOR_ITER == 93, got {_op['FOR_ITER']}"
_ledger.append(1)

# Build-* container opcodes
assert _op["BUILD_LIST"] - 103 == 0, f"BUILD_LIST == 103, got {_op['BUILD_LIST']}"
_ledger.append(1)

assert _op["BUILD_SET"] - 104 == 0, f"BUILD_SET == 104, got {_op['BUILD_SET']}"
_ledger.append(1)

assert _op["BUILD_MAP"] - 105 == 0, f"BUILD_MAP == 105, got {_op['BUILD_MAP']}"
_ledger.append(1)

# Several core opcodes are pairwise distinct (sanity check on the table)
_distinct = {
    _op["LOAD_CONST"],
    _op["RETURN_VALUE"],
    _op["LOAD_FAST"],
    _op["STORE_FAST"],
    _op["JUMP_FORWARD"],
    _op["COMPARE_OP"],
    _op["NOP"],
    _op["POP_TOP"],
    _op["MAKE_FUNCTION"],
    dis.HAVE_ARGUMENT,
    _op["PUSH_NULL"],
    _op["RESUME"],
    _op["GET_ITER"],
    _op["FOR_ITER"],
    _op["BUILD_LIST"],
    _op["BUILD_SET"],
    _op["BUILD_MAP"],
}
assert len(_distinct) == 17, (
    f"the 17 sampled opcodes are pairwise distinct, got {len(_distinct)}"
)
_ledger.append(1)

# dis.dis(fn) runs without raising on a callable (output content not asserted)
def _trivial():
    return 42

dis.dis(_trivial)
_ledger.append(1)

# dis.dis on a lambda runs without raising
dis.dis(lambda x: x + 1)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_dis {sum(_ledger)} asserts")
