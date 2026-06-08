# Atomic 288 pass conformance — array module (hasattr array/
# typecodes + typecodes 'bBuhHiIlLqQfd' + array('i', [1,2,3]).
# tolist/typecode/itemsize) + struct module (hasattr pack/unpack/
# calcsize/pack_into/unpack_from/iter_unpack/Struct/error + pack
# returns bytes + calcsize('i')==4 + calcsize('b')==1 +
# calcsize('h')==2 + calcsize('q')==8 + calcsize('I')==4 + unpack
# returns tuple + round-trip 'i'/'q') + binascii module (hasattr
# hexlify/unhexlify/a2b_hex/b2a_hex/a2b_base64/b2a_base64 + hexlify/
# unhexlify round-trip) + zlib module (hasattr compress/decompress/
# crc32/adler32 + crc32(b'hello')==907060870 + adler32(b'hello')
# ==103547413 + compress+decompress round-trip).
# All asserts match between CPython 3.12 and mamba.
import array
import struct
import binascii
import zlib


_ledger: list[int] = []

# 1) array — hasattr surface
assert hasattr(array, "array") == True; _ledger.append(1)
assert hasattr(array, "typecodes") == True; _ledger.append(1)

# 2) array — typecodes value contract
assert array.typecodes == "bBuhHiIlLqQfd"; _ledger.append(1)

# 3) array — instance attribute access
_a = array.array("i", [1, 2, 3])
assert _a.tolist() == [1, 2, 3]; _ledger.append(1)
assert _a.typecode == "i"; _ledger.append(1)
assert _a.itemsize == 4; _ledger.append(1)

# 4) struct — hasattr surface
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)
assert hasattr(struct, "iter_unpack") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)
assert hasattr(struct, "error") == True; _ledger.append(1)

# 5) struct — pack returns bytes
assert isinstance(struct.pack("i", 5), bytes) == True; _ledger.append(1)

# 6) struct — calcsize value contracts
assert struct.calcsize("i") == 4; _ledger.append(1)
assert struct.calcsize("b") == 1; _ledger.append(1)
assert struct.calcsize("h") == 2; _ledger.append(1)
assert struct.calcsize("q") == 8; _ledger.append(1)
assert struct.calcsize("I") == 4; _ledger.append(1)

# 7) struct — unpack returns tuple
assert isinstance(struct.unpack("i", struct.pack("i", 5)), tuple) == True; _ledger.append(1)

# 8) struct — round-trip value contracts
assert struct.unpack("i", struct.pack("i", 5))[0] == 5; _ledger.append(1)
assert struct.unpack("q", struct.pack("q", 123))[0] == 123; _ledger.append(1)

# 9) binascii — hasattr core surface
assert hasattr(binascii, "hexlify") == True; _ledger.append(1)
assert hasattr(binascii, "unhexlify") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_hex") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_hex") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_base64") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_base64") == True; _ledger.append(1)

# 10) binascii — hexlify/unhexlify round-trip
assert binascii.hexlify(b"abc") == b"616263"; _ledger.append(1)
assert binascii.unhexlify("616263") == b"abc"; _ledger.append(1)

# 11) zlib — hasattr core surface
assert hasattr(zlib, "compress") == True; _ledger.append(1)
assert hasattr(zlib, "decompress") == True; _ledger.append(1)
assert hasattr(zlib, "crc32") == True; _ledger.append(1)
assert hasattr(zlib, "adler32") == True; _ledger.append(1)

# 12) zlib — checksum value contracts
assert zlib.crc32(b"hello") == 907060870; _ledger.append(1)
assert zlib.adler32(b"hello") == 103547413; _ledger.append(1)

# 13) zlib — compress/decompress round-trip
assert zlib.decompress(zlib.compress(b"hello world")) == b"hello world"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_array_struct_binascii_zlib_value_ops {sum(_ledger)} asserts")
