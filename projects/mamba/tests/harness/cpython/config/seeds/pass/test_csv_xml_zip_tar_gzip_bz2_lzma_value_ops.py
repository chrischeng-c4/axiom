# Operational AssertionPass seed for the value contract of the
# `io` / `csv` / `configparser` / `xml.etree.ElementTree` /
# `zipfile` / `tarfile` / `gzip` / `bz2` / `lzma` nine-pack
# pinned to atomic 220: `io` (the documented partial module-
# level class identifier hasattr surface — `StringIO` /
# `BytesIO`), `csv` (the documented partial module-level
# helper / class / sentinel identifier hasattr surface —
# `reader` / `writer` / `DictReader` / `DictWriter` /
# `Dialect` / `excel` / `excel_tab` / `unix_dialect` /
# `QUOTE_ALL` / `QUOTE_MINIMAL` / `QUOTE_NONNUMERIC` /
# `QUOTE_NONE` / `register_dialect` / `unregister_dialect`
# / `list_dialects` / `get_dialect` / `field_size_limit` /
# `Error`), `configparser` (the documented partial module-
# level class identifier hasattr surface — `ConfigParser`),
# `xml.etree.ElementTree` (the documented partial module-
# level helper / class identifier hasattr surface —
# `Element` / `SubElement` / `ElementTree` / `parse` /
# `fromstring` / `tostring` / `iselement` / `Comment` /
# `PI` / `ProcessingInstruction` / `register_namespace` /
# `TreeBuilder` / `QName` + the documented
# `ET.fromstring("<root><child>data</child></root>").tag
# == "root"` xml-parse tag value contract), `zipfile`
# (the documented partial module-level helper / class /
# sentinel identifier hasattr surface — `ZipFile` /
# `is_zipfile` / `ZIP_STORED` / `ZIP_DEFLATED`),
# `tarfile` (the documented partial module-level helper
# identifier hasattr surface — `is_tarfile` / `open`),
# `gzip` (the documented partial module-level helper /
# class / exception identifier hasattr surface —
# `GzipFile` / `compress` / `decompress` / `open` /
# `BadGzipFile` + the documented
# `type(gzip.compress(b"hello")).__name__ == "bytes"` /
# `gzip.decompress(gzip.compress(b"hello")) == b"hello"`
# gzip codec roundtrip value contract), `bz2` (the
# documented full module-level helper / class identifier
# hasattr surface — `BZ2File` / `BZ2Compressor` /
# `BZ2Decompressor` / `compress` / `decompress` / `open`
# + the documented
# `type(bz2.compress(b"hello")).__name__ == "bytes"` /
# `bz2.decompress(bz2.compress(b"hello")) == b"hello"`
# bz2 codec roundtrip value contract), and `lzma` (the
# documented full module-level helper / class / sentinel
# identifier hasattr surface — `LZMAFile` /
# `LZMACompressor` / `LZMADecompressor` / `compress` /
# `decompress` / `open` / `FORMAT_XZ` / `FORMAT_ALONE` /
# `FORMAT_RAW` + the documented
# `type(lzma.compress(b"hello")).__name__ == "bytes"` /
# `lzma.decompress(lzma.compress(b"hello")) == b"hello"`
# lzma codec roundtrip value contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(io, "TextIOWrapper") / "BufferedReader" /
# "BufferedWriter" / "BufferedRandom" / "BufferedRWPair"
# / "BufferedIOBase" / "RawIOBase" / "TextIOBase" /
# "IOBase" / "FileIO" / "open" / "DEFAULT_BUFFER_SIZE" /
# "SEEK_SET" / "SEEK_CUR" / "SEEK_END" /
# "UnsupportedOperation" all False on mamba +
# type(io.StringIO("hello")).__name__ == "StringIO"
# collapses to "dict" on mamba, hasattr(csv, "Sniffer")
# False on mamba, hasattr(configparser,
# "RawConfigParser") / "BasicInterpolation" /
# "ExtendedInterpolation" / "NoSectionError" /
# "NoOptionError" / "DuplicateSectionError" /
# "DuplicateOptionError" / "InterpolationError" /
# "MissingSectionHeaderError" / "ParsingError" /
# "Error" / "DEFAULTSECT" all False on mamba +
# type(configparser.ConfigParser()).__name__ ==
# "ConfigParser" collapses to "dict" on mamba,
# hasattr(ET, "XMLParser") / "XMLPullParser" /
# "VERSION" all False on mamba +
# type(ET.fromstring("<r><c>d</c></r>")).__name__ ==
# "Element" collapses to "dict" + len(ET.fromstring(
# "<root><child>data</child></root>")) == 1 collapses
# to 6 on mamba, hasattr(zipfile, "ZipInfo") /
# "BadZipFile" / "BadZipfile" / "LargeZipFile" /
# "ZIP_BZIP2" / "ZIP_LZMA" / "Path" all False on mamba,
# hasattr(tarfile, "TarFile") / "TarInfo" / "TarError"
# / "ReadError" / "CompressionError" / "StreamError" /
# "ExtractError" / "USTAR_FORMAT" / "GNU_FORMAT" /
# "PAX_FORMAT" all False on mamba, hasattr(gzip,
# "READ") / "WRITE" all False on mamba) are covered in
# the matching spec fixture `lang_io_configparser_et_silent`.
import io
import csv
import configparser
import xml.etree.ElementTree as ET
import zipfile
import tarfile
import gzip
import bz2
import lzma


_ledger: list[int] = []

# 1) io — partial module hasattr surface
#    (16 attrs + type(io.StringIO()).__name__ DIVERGE
#    on mamba — moved to spec)
assert hasattr(io, "StringIO") == True; _ledger.append(1)
assert hasattr(io, "BytesIO") == True; _ledger.append(1)

# 2) csv — partial module hasattr surface
#    (Sniffer DIVERGE on mamba — moved to spec)
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "excel") == True; _ledger.append(1)
assert hasattr(csv, "excel_tab") == True; _ledger.append(1)
assert hasattr(csv, "unix_dialect") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "register_dialect") == True; _ledger.append(1)
assert hasattr(csv, "unregister_dialect") == True; _ledger.append(1)
assert hasattr(csv, "list_dialects") == True; _ledger.append(1)
assert hasattr(csv, "get_dialect") == True; _ledger.append(1)
assert hasattr(csv, "field_size_limit") == True; _ledger.append(1)
assert hasattr(csv, "Error") == True; _ledger.append(1)

# 3) configparser — partial module hasattr surface
#    (12 attrs + type(ConfigParser()).__name__ DIVERGE
#    on mamba — moved to spec)
assert hasattr(configparser, "ConfigParser") == True; _ledger.append(1)

# 4) xml.etree.ElementTree — partial module hasattr
#    surface (XMLParser / XMLPullParser / VERSION
#    DIVERGE on mamba — moved to spec)
assert hasattr(ET, "Element") == True; _ledger.append(1)
assert hasattr(ET, "SubElement") == True; _ledger.append(1)
assert hasattr(ET, "ElementTree") == True; _ledger.append(1)
assert hasattr(ET, "parse") == True; _ledger.append(1)
assert hasattr(ET, "fromstring") == True; _ledger.append(1)
assert hasattr(ET, "tostring") == True; _ledger.append(1)
assert hasattr(ET, "iselement") == True; _ledger.append(1)
assert hasattr(ET, "Comment") == True; _ledger.append(1)
assert hasattr(ET, "PI") == True; _ledger.append(1)
assert hasattr(ET, "ProcessingInstruction") == True; _ledger.append(1)
assert hasattr(ET, "register_namespace") == True; _ledger.append(1)
assert hasattr(ET, "TreeBuilder") == True; _ledger.append(1)
assert hasattr(ET, "QName") == True; _ledger.append(1)

# 5) xml.etree.ElementTree — xml-parse tag value contract
#    (type(fromstring()).__name__ and len(fromstring())
#    DIVERGE on mamba — moved to spec)
_root = ET.fromstring("<root><child>data</child></root>")
assert _root.tag == "root"; _ledger.append(1)

# 6) zipfile — partial module hasattr surface
#    (7 attrs DIVERGE on mamba — moved to spec)
assert hasattr(zipfile, "ZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "is_zipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_STORED") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_DEFLATED") == True; _ledger.append(1)

# 7) tarfile — partial module hasattr surface
#    (10 attrs DIVERGE on mamba — moved to spec)
assert hasattr(tarfile, "is_tarfile") == True; _ledger.append(1)
assert hasattr(tarfile, "open") == True; _ledger.append(1)

# 8) gzip — partial module hasattr surface
#    (READ / WRITE DIVERGE on mamba — moved to spec)
assert hasattr(gzip, "GzipFile") == True; _ledger.append(1)
assert hasattr(gzip, "compress") == True; _ledger.append(1)
assert hasattr(gzip, "decompress") == True; _ledger.append(1)
assert hasattr(gzip, "open") == True; _ledger.append(1)
assert hasattr(gzip, "BadGzipFile") == True; _ledger.append(1)

# 9) gzip — gzip codec roundtrip value contract
_g = gzip.compress(b"hello")
assert type(_g).__name__ == "bytes"; _ledger.append(1)
assert gzip.decompress(_g) == b"hello"; _ledger.append(1)

# 10) bz2 — full module hasattr surface
assert hasattr(bz2, "BZ2File") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Compressor") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Decompressor") == True; _ledger.append(1)
assert hasattr(bz2, "compress") == True; _ledger.append(1)
assert hasattr(bz2, "decompress") == True; _ledger.append(1)
assert hasattr(bz2, "open") == True; _ledger.append(1)

# 11) bz2 — bz2 codec roundtrip value contract
_b = bz2.compress(b"hello")
assert type(_b).__name__ == "bytes"; _ledger.append(1)
assert bz2.decompress(_b) == b"hello"; _ledger.append(1)

# 12) lzma — full module hasattr surface
assert hasattr(lzma, "LZMAFile") == True; _ledger.append(1)
assert hasattr(lzma, "LZMACompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMADecompressor") == True; _ledger.append(1)
assert hasattr(lzma, "compress") == True; _ledger.append(1)
assert hasattr(lzma, "decompress") == True; _ledger.append(1)
assert hasattr(lzma, "open") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_XZ") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_ALONE") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_RAW") == True; _ledger.append(1)

# 13) lzma — lzma codec roundtrip value contract
_l = lzma.compress(b"hello")
assert type(_l).__name__ == "bytes"; _ledger.append(1)
assert lzma.decompress(_l) == b"hello"; _ledger.append(1)

# NB: hasattr(io, "TextIOWrapper") / "BufferedReader" /
# "BufferedWriter" / "BufferedRandom" / "BufferedRWPair"
# / "BufferedIOBase" / "RawIOBase" / "TextIOBase" /
# "IOBase" / "FileIO" / "open" / "DEFAULT_BUFFER_SIZE"
# / "SEEK_SET" / "SEEK_CUR" / "SEEK_END" /
# "UnsupportedOperation" all False on mamba +
# type(io.StringIO("hello")).__name__ == "StringIO"
# collapses to "dict" on mamba, hasattr(csv, "Sniffer")
# False on mamba, hasattr(configparser,
# "RawConfigParser") / "BasicInterpolation" /
# "ExtendedInterpolation" / "NoSectionError" /
# "NoOptionError" / "DuplicateSectionError" /
# "DuplicateOptionError" / "InterpolationError" /
# "MissingSectionHeaderError" / "ParsingError" /
# "Error" / "DEFAULTSECT" all False on mamba +
# type(configparser.ConfigParser()).__name__ ==
# "ConfigParser" collapses to "dict" on mamba,
# hasattr(ET, "XMLParser") / "XMLPullParser" /
# "VERSION" all False on mamba +
# type(ET.fromstring("<r><c>d</c></r>")).__name__ ==
# "Element" collapses to "dict" + len(ET.fromstring(
# "<root><child>data</child></root>")) == 1 collapses
# to 6 on mamba, hasattr(zipfile, "ZipInfo") /
# "BadZipFile" / "BadZipfile" / "LargeZipFile" /
# "ZIP_BZIP2" / "ZIP_LZMA" / "Path" all False on mamba,
# hasattr(tarfile, "TarFile") / "TarInfo" / "TarError"
# / "ReadError" / "CompressionError" / "StreamError" /
# "ExtractError" / "USTAR_FORMAT" / "GNU_FORMAT" /
# "PAX_FORMAT" all False on mamba, hasattr(gzip, "READ")
# / "WRITE" all False on mamba — all DIVERGE on mamba
# — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_csv_xml_zip_tar_gzip_bz2_lzma_value_ops {sum(_ledger)} asserts")
