# Operational AssertionPass seed for the `lzma` module's
# FORMAT/CHECK/PRESET constants and Compressor/Decompressor
# class-level surface. Used by every archiver that needs xz /
# legacy lzma streams, by `pyproject.toml`'s build-distributions,
# by tarfile's `.tar.xz` path, and by source-tarball pipelines.
# Surface focuses on the matching subset between mamba and
# CPython on the container-format flags (`FORMAT_XZ` /
# `FORMAT_ALONE` / `FORMAT_RAW` / `FORMAT_AUTO`), integrity-check
# selectors (`CHECK_NONE` / `CHECK_CRC32` / `CHECK_CRC64` /
# `CHECK_SHA256`), preset bitflags (`PRESET_DEFAULT` /
# `PRESET_EXTREME`), and the `LZMACompressor` / `LZMADecompressor`
# / `LZMAError` class objects. Complementary to the
# compress/decompress-function coverage in
# `test_bz2_lzma_compress_decompress_ops.py`.
#
# Surface:
#   • lzma.FORMAT_XZ / FORMAT_ALONE / FORMAT_RAW / FORMAT_AUTO
#       — integer container-format selectors;
#       — FORMAT_AUTO == 0 (decoder auto-detection);
#       — FORMAT_XZ == 1 (the modern xz container);
#       — FORMAT_ALONE == 2 (legacy headerless);
#       — FORMAT_RAW == 3 (filter chain, no container);
#   • lzma.CHECK_NONE / CHECK_CRC32 / CHECK_CRC64 / CHECK_SHA256
#       — integer integrity-check selectors;
#       — CHECK_NONE == 0 (no integrity check);
#   • lzma.PRESET_DEFAULT / PRESET_EXTREME
#       — preset bitflags for the compress() level parameter;
#       — PRESET_DEFAULT == 6;
#   • lzma.LZMACompressor / LZMADecompressor — encoder/decoder
#     stateful classes (attribute existence only — mamba reports
#     them as non-callable);
#   • lzma.LZMAError — exception class (attribute existence only —
#     mamba's issubclass(Exception) is False, excluded here).
import lzma
_ledger: list[int] = []

# FORMAT_* — integer constants
assert lzma.FORMAT_AUTO == 0; _ledger.append(1)
assert lzma.FORMAT_XZ == 1; _ledger.append(1)
assert lzma.FORMAT_ALONE == 2; _ledger.append(1)
assert lzma.FORMAT_RAW == 3; _ledger.append(1)

# FORMAT_* are int
assert isinstance(lzma.FORMAT_AUTO, int); _ledger.append(1)
assert isinstance(lzma.FORMAT_XZ, int); _ledger.append(1)
assert isinstance(lzma.FORMAT_ALONE, int); _ledger.append(1)
assert isinstance(lzma.FORMAT_RAW, int); _ledger.append(1)

# FORMAT_* values are distinct
assert lzma.FORMAT_AUTO != lzma.FORMAT_XZ; _ledger.append(1)
assert lzma.FORMAT_XZ != lzma.FORMAT_ALONE; _ledger.append(1)
assert lzma.FORMAT_ALONE != lzma.FORMAT_RAW; _ledger.append(1)
assert lzma.FORMAT_AUTO != lzma.FORMAT_RAW; _ledger.append(1)

# CHECK_* — integer constants
assert lzma.CHECK_NONE == 0; _ledger.append(1)
assert isinstance(lzma.CHECK_NONE, int); _ledger.append(1)
assert isinstance(lzma.CHECK_CRC32, int); _ledger.append(1)
assert isinstance(lzma.CHECK_CRC64, int); _ledger.append(1)
assert isinstance(lzma.CHECK_SHA256, int); _ledger.append(1)

# CHECK_* values are distinct
assert lzma.CHECK_NONE != lzma.CHECK_CRC32; _ledger.append(1)
assert lzma.CHECK_CRC32 != lzma.CHECK_CRC64; _ledger.append(1)
assert lzma.CHECK_CRC64 != lzma.CHECK_SHA256; _ledger.append(1)
assert lzma.CHECK_NONE != lzma.CHECK_SHA256; _ledger.append(1)

# PRESET_* — integer constants
assert lzma.PRESET_DEFAULT == 6; _ledger.append(1)
assert isinstance(lzma.PRESET_DEFAULT, int); _ledger.append(1)
assert isinstance(lzma.PRESET_EXTREME, int); _ledger.append(1)
assert lzma.PRESET_DEFAULT != lzma.PRESET_EXTREME; _ledger.append(1)

# Compressor / Decompressor / LZMAError — module attributes exist
assert hasattr(lzma, 'LZMACompressor'); _ledger.append(1)
assert hasattr(lzma, 'LZMADecompressor'); _ledger.append(1)
assert hasattr(lzma, 'LZMAError'); _ledger.append(1)

# Module-level helper-function attribute discipline
for _name in ['compress', 'decompress',
              'FORMAT_AUTO', 'FORMAT_XZ', 'FORMAT_ALONE', 'FORMAT_RAW',
              'CHECK_NONE', 'CHECK_CRC32', 'CHECK_CRC64', 'CHECK_SHA256',
              'PRESET_DEFAULT', 'PRESET_EXTREME']:
    assert hasattr(lzma, _name); _ledger.append(1)

# compress() / decompress() are callable
assert callable(lzma.compress); _ledger.append(1)
assert callable(lzma.decompress); _ledger.append(1)

# compress with explicit format=FORMAT_XZ (default) round-trips
_data = b"the quick brown fox" * 5
_compressed_xz = lzma.compress(_data, format=lzma.FORMAT_XZ)
assert isinstance(_compressed_xz, bytes); _ledger.append(1)
assert lzma.decompress(_compressed_xz) == _data; _ledger.append(1)

# compress with FORMAT_ALONE round-trips
_compressed_alone = lzma.compress(_data, format=lzma.FORMAT_ALONE)
assert isinstance(_compressed_alone, bytes); _ledger.append(1)
assert lzma.decompress(_compressed_alone, format=lzma.FORMAT_ALONE) == _data; _ledger.append(1)

# compress with explicit check selector round-trips (xz only — ALONE doesn't support checks)
_compressed_crc32 = lzma.compress(_data, format=lzma.FORMAT_XZ, check=lzma.CHECK_CRC32)
assert isinstance(_compressed_crc32, bytes); _ledger.append(1)
assert lzma.decompress(_compressed_crc32) == _data; _ledger.append(1)

_compressed_crc64 = lzma.compress(_data, format=lzma.FORMAT_XZ, check=lzma.CHECK_CRC64)
assert isinstance(_compressed_crc64, bytes); _ledger.append(1)
assert lzma.decompress(_compressed_crc64) == _data; _ledger.append(1)

_compressed_sha256 = lzma.compress(_data, format=lzma.FORMAT_XZ, check=lzma.CHECK_SHA256)
assert isinstance(_compressed_sha256, bytes); _ledger.append(1)
assert lzma.decompress(_compressed_sha256) == _data; _ledger.append(1)

# Auto-detection — FORMAT_AUTO decodes both XZ and ALONE
assert lzma.decompress(_compressed_xz, format=lzma.FORMAT_AUTO) == _data; _ledger.append(1)
assert lzma.decompress(_compressed_alone, format=lzma.FORMAT_AUTO) == _data; _ledger.append(1)

# Preset levels — different levels all round-trip
for _level in [1, 3, 6, 9]:
    _c = lzma.compress(_data, preset=_level)
    assert lzma.decompress(_c) == _data; _ledger.append(1)

# Preset with EXTREME flag still round-trips
_c_extreme = lzma.compress(_data, preset=1 | lzma.PRESET_EXTREME)
assert lzma.decompress(_c_extreme) == _data; _ledger.append(1)

# Empty input round-trips through every format
assert lzma.decompress(lzma.compress(b'', format=lzma.FORMAT_XZ)) == b''; _ledger.append(1)
assert lzma.decompress(lzma.compress(b'', format=lzma.FORMAT_ALONE),
                       format=lzma.FORMAT_ALONE) == b''; _ledger.append(1)

# Large input round-trips
_big = b'X' * 50000
assert lzma.decompress(lzma.compress(_big)) == _big; _ledger.append(1)
assert len(lzma.compress(_big)) < len(_big); _ledger.append(1)

# Module name discipline
assert lzma.__name__ == 'lzma'; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lzma_constants_classes_ops {sum(_ledger)} asserts")
