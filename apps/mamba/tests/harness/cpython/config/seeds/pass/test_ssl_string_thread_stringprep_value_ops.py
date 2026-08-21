# Atomic 311 pass conformance — ssl module (hasattr SSLContext/SSL
# Socket/SSLError/CERT_NONE/CERT_OPTIONAL/CERT_REQUIRED/PROTOCOL_TLS/
# PROTOCOL_TLS_CLIENT/PROTOCOL_TLS_SERVER/Purpose/create_default_
# context/OP_ALL/OP_NO_SSLv2/OP_NO_SSLv3/VERIFY_DEFAULT + CERT_NONE ==
# 0 + CERT_OPTIONAL == 1 + CERT_REQUIRED == 2) + string module
# (hasattr ascii_letters/ascii_lowercase/ascii_uppercase/digits/hex
# digits/octdigits/punctuation/whitespace/Formatter/Template/capwords
# + ascii_lowercase + digits + hexdigits + octdigits + capwords). +
# _thread module (hasattr start_new_thread/allocate_lock/get_ident/
# LockType/exit/error/interrupt_main + isinstance(get_ident(), int))
# + stringprep module (hasattr in_table_a1/in_table_b1/in_table_c11/
# in_table_c12/in_table_c21/in_table_c22/in_table_d1/in_table_d2 +
# in_table_a1('a') False + in_table_c11(' ') True).
# All asserts match between CPython 3.12 and mamba.
import ssl
import string
import _thread
import stringprep


_ledger: list[int] = []

# 1) ssl — hasattr core surface
assert hasattr(ssl, "SSLContext") == True; _ledger.append(1)
assert hasattr(ssl, "SSLSocket") == True; _ledger.append(1)
assert hasattr(ssl, "SSLError") == True; _ledger.append(1)
assert hasattr(ssl, "CERT_NONE") == True; _ledger.append(1)
assert hasattr(ssl, "CERT_OPTIONAL") == True; _ledger.append(1)
assert hasattr(ssl, "CERT_REQUIRED") == True; _ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS") == True; _ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS_CLIENT") == True; _ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS_SERVER") == True; _ledger.append(1)
assert hasattr(ssl, "Purpose") == True; _ledger.append(1)
assert hasattr(ssl, "create_default_context") == True; _ledger.append(1)
assert hasattr(ssl, "OP_ALL") == True; _ledger.append(1)
assert hasattr(ssl, "OP_NO_SSLv2") == True; _ledger.append(1)
assert hasattr(ssl, "OP_NO_SSLv3") == True; _ledger.append(1)
assert hasattr(ssl, "VERIFY_DEFAULT") == True; _ledger.append(1)

# 2) ssl — value contracts (VerifyMode IntEnum compares to int)
assert ssl.CERT_NONE == 0; _ledger.append(1)
assert ssl.CERT_OPTIONAL == 1; _ledger.append(1)
assert ssl.CERT_REQUIRED == 2; _ledger.append(1)

# 3) string — hasattr core surface (conformant subset)
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)

# 4) string — value contracts
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)

# 5) _thread — hasattr core surface + int identity
assert hasattr(_thread, "start_new_thread") == True; _ledger.append(1)
assert hasattr(_thread, "allocate_lock") == True; _ledger.append(1)
assert hasattr(_thread, "get_ident") == True; _ledger.append(1)
assert hasattr(_thread, "LockType") == True; _ledger.append(1)
assert hasattr(_thread, "exit") == True; _ledger.append(1)
assert hasattr(_thread, "error") == True; _ledger.append(1)
assert hasattr(_thread, "interrupt_main") == True; _ledger.append(1)
assert isinstance(_thread.get_ident(), int) == True; _ledger.append(1)

# 6) stringprep — hasattr core surface + value contracts
assert hasattr(stringprep, "in_table_a1") == True; _ledger.append(1)
assert hasattr(stringprep, "in_table_b1") == True; _ledger.append(1)
assert hasattr(stringprep, "in_table_c11") == True; _ledger.append(1)
assert hasattr(stringprep, "in_table_c12") == True; _ledger.append(1)
assert hasattr(stringprep, "in_table_c21") == True; _ledger.append(1)
assert hasattr(stringprep, "in_table_c22") == True; _ledger.append(1)
assert hasattr(stringprep, "in_table_d1") == True; _ledger.append(1)
assert hasattr(stringprep, "in_table_d2") == True; _ledger.append(1)
assert stringprep.in_table_a1("a") == False; _ledger.append(1)
assert stringprep.in_table_c11(" ") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_ssl_string_thread_stringprep_value_ops {sum(_ledger)} asserts")
