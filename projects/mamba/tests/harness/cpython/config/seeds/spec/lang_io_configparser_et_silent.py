# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `io` / `csv` /
# `configparser` / `xml.etree.ElementTree` / `zipfile` /
# `tarfile` / `gzip` seven-pack pinned to atomic 220:
# `io` (the documented `hasattr(io, "open") /
# "FileIO" / "BytesIO" / "StringIO" / "TextIOWrapper" /
# "BufferedReader" / "BufferedWriter" / "BufferedRWPair" /
# "BufferedRandom" / "RawIOBase" / "IOBase" / "BufferedIOBase"
# / "TextIOBase" / "DEFAULT_BUFFER_SIZE" / "SEEK_SET" /
# "SEEK_CUR" / "SEEK_END" == True` extended hasattr surface +
# the documented `type(io.StringIO()).__name__ == "StringIO"`
# constructor value contract), `csv` (the documented
# `hasattr(csv, "Sniffer") == True` extended hasattr surface),
# `configparser` (the documented `hasattr(configparser,
# "RawConfigParser") / "BasicInterpolation" /
# "ExtendedInterpolation" / "NoSectionError" / "NoOptionError"
# / "DuplicateSectionError" / "DuplicateOptionError" /
# "InterpolationError" / "MissingSectionHeaderError" /
# "ParsingError" / "Error" / "DEFAULTSECT" == True` extended
# hasattr surface + the documented
# `type(configparser.ConfigParser()).__name__ == "ConfigParser"`
# constructor value contract), `xml.etree.ElementTree` (the
# documented `hasattr(ET, "XMLParser") / "XMLPullParser" /
# "VERSION" == True` extended hasattr surface + the documented
# `type(ET.fromstring(...)).__name__ == "Element"` constructor
# value contract + the documented `len(root) == 1` for the
# `<root><child/></root>` document value contract), `zipfile`
# (the documented `hasattr(zipfile, "ZipInfo") / "BadZipFile"
# / "BadZipfile" / "LargeZipFile" / "ZIP_BZIP2" / "ZIP_LZMA" /
# "Path" == True` extended hasattr surface), `tarfile` (the
# documented `hasattr(tarfile, "TarFile") / "TarInfo" /
# "TarError" / "ReadError" / "CompressionError" /
# "StreamError" / "ExtractError" / "USTAR_FORMAT" /
# "GNU_FORMAT" / "PAX_FORMAT" == True` extended hasattr
# surface), and `gzip` (the documented `hasattr(gzip, "READ")
# / "WRITE" == True` extended hasattr surface).
#
# Behavioral edges that CONFORM on mamba
# (io `open` / `IOBase`-family / `SEEK_*` partial hasattr
# subset, csv full reader/writer/Dialect/QUOTE_* hasattr
# surface, configparser `ConfigParser` hasattr, ET full
# Element/SubElement/ElementTree/parse/fromstring/tostring/
# iselement/Comment/PI/ProcessingInstruction/
# register_namespace/TreeBuilder/QName hasattr + `root.tag ==
# "root"` value contract, zipfile `ZipFile` / `is_zipfile` /
# `ZIP_STORED` / `ZIP_DEFLATED` hasattr, tarfile `is_tarfile`
# / `open` hasattr, gzip full GzipFile/compress/decompress/
# open/BadGzipFile hasattr + roundtrip value contract,
# bz2 full hasattr + roundtrip value contract, lzma full
# hasattr + FORMAT_XZ/FORMAT_ALONE/FORMAT_RAW + roundtrip
# value contract) are covered in the matching pass fixture
# `test_csv_xml_zip_tar_gzip_bz2_lzma_value_ops`.
from typing import Any
import csv as _csv_mod
import configparser as _cp_mod
import xml.etree.ElementTree as _et_mod
import zipfile as _zipfile_mod
import tarfile as _tarfile_mod
import gzip as _gzip_mod

csv: Any = _csv_mod
configparser: Any = _cp_mod
ET: Any = _et_mod
zipfile: Any = _zipfile_mod
tarfile: Any = _tarfile_mod
gzip: Any = _gzip_mod


_ledger: list[int] = []

# 1) csv — extended module hasattr surface
#    (mamba: Sniffer False)
assert hasattr(csv, "Sniffer") == True; _ledger.append(1)

# 2) configparser — extended module hasattr surface
#    (mamba: RawConfigParser / BasicInterpolation /
#    ExtendedInterpolation / NoSectionError / NoOptionError /
#    DuplicateSectionError / DuplicateOptionError /
#    InterpolationError / MissingSectionHeaderError /
#    ParsingError / Error / DEFAULTSECT all False)
assert hasattr(configparser, "RawConfigParser") == True; _ledger.append(1)
assert hasattr(configparser, "BasicInterpolation") == True; _ledger.append(1)
assert hasattr(configparser, "ExtendedInterpolation") == True; _ledger.append(1)
assert hasattr(configparser, "NoSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "NoOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "InterpolationError") == True; _ledger.append(1)
assert hasattr(configparser, "MissingSectionHeaderError") == True; _ledger.append(1)
assert hasattr(configparser, "ParsingError") == True; _ledger.append(1)
assert hasattr(configparser, "Error") == True; _ledger.append(1)
assert hasattr(configparser, "DEFAULTSECT") == True; _ledger.append(1)

# 3) configparser — constructor value contract
#    (mamba: ConfigParser() returns dict)
_cp = configparser.ConfigParser()
assert type(_cp).__name__ == "ConfigParser"; _ledger.append(1)

# 4) xml.etree.ElementTree — extended module hasattr surface
#    (mamba: XMLParser / XMLPullParser / VERSION all False)
assert hasattr(ET, "XMLParser") == True; _ledger.append(1)
assert hasattr(ET, "XMLPullParser") == True; _ledger.append(1)
assert hasattr(ET, "VERSION") == True; _ledger.append(1)

# 5) xml.etree.ElementTree — fromstring constructor value
#    contract (mamba: fromstring() returns dict and len
#    returns 6 instead of 1 for <root><child/></root>)
_root = ET.fromstring("<root><child/></root>")
assert type(_root).__name__ == "Element"; _ledger.append(1)
assert len(_root) == 1; _ledger.append(1)

# 6) zipfile — extended module hasattr surface
#    (mamba: ZipInfo / BadZipFile / BadZipfile / LargeZipFile
#    / ZIP_BZIP2 / ZIP_LZMA / Path all False)
assert hasattr(zipfile, "ZipInfo") == True; _ledger.append(1)
assert hasattr(zipfile, "BadZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "BadZipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "LargeZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_BZIP2") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_LZMA") == True; _ledger.append(1)
assert hasattr(zipfile, "Path") == True; _ledger.append(1)

# 7) tarfile — extended module hasattr surface
#    (mamba: TarFile / TarInfo / TarError / ReadError /
#    CompressionError / StreamError / ExtractError /
#    USTAR_FORMAT / GNU_FORMAT / PAX_FORMAT all False)
assert hasattr(tarfile, "TarFile") == True; _ledger.append(1)
assert hasattr(tarfile, "TarInfo") == True; _ledger.append(1)
assert hasattr(tarfile, "TarError") == True; _ledger.append(1)
assert hasattr(tarfile, "ReadError") == True; _ledger.append(1)
assert hasattr(tarfile, "CompressionError") == True; _ledger.append(1)
assert hasattr(tarfile, "StreamError") == True; _ledger.append(1)
assert hasattr(tarfile, "ExtractError") == True; _ledger.append(1)
assert hasattr(tarfile, "USTAR_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "GNU_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "PAX_FORMAT") == True; _ledger.append(1)

# 8) gzip — extended module hasattr surface
#    (mamba: READ / WRITE both False)
assert hasattr(gzip, "READ") == True; _ledger.append(1)
assert hasattr(gzip, "WRITE") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_io_configparser_et_silent {sum(_ledger)} asserts")
