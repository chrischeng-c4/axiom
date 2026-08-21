# Operational AssertionPass seed for `mimetypes.guess_extension`
# on canonical image and binary types, and the double-extension
# `guess_type("file.tar.gz")` form. Surface: image/png ↔ .png,
# image/jpeg ↔ .jpg or .jpeg, application/json ↔ .json,
# application/zip is detected from .zip, application/pdf is
# detected from .pdf. The compressed-archive double extension
# `file.tar.gz` returns the inner type as application/x-tar and
# the second element of the tuple as the gzip encoding indicator.
import mimetypes
_ledger: list[int] = []

# guess_extension on common types
assert mimetypes.guess_extension("image/png") == ".png"; _ledger.append(1)
assert mimetypes.guess_extension("application/json") == ".json"; _ledger.append(1)

# image/jpeg accepts either .jpg or .jpeg per Python version
ext_jpeg = mimetypes.guess_extension("image/jpeg")
assert ext_jpeg in (".jpg", ".jpeg"); _ledger.append(1)

# guess_type detects image/binary types
assert mimetypes.guess_type("file.png")[0] == "image/png"; _ledger.append(1)
assert mimetypes.guess_type("file.jpg")[0] == "image/jpeg"; _ledger.append(1)
assert mimetypes.guess_type("file.pdf")[0] == "application/pdf"; _ledger.append(1)
assert mimetypes.guess_type("file.zip")[0] == "application/zip"; _ledger.append(1)
assert mimetypes.guess_type("file.json")[0] == "application/json"; _ledger.append(1)

# Double-extension: file.tar.gz returns (x-tar, gzip)
tg = mimetypes.guess_type("file.tar.gz")
assert tg[0] == "application/x-tar"; _ledger.append(1)
assert tg[1] == "gzip"; _ledger.append(1)

# Single-extension archive returns no encoding
assert mimetypes.guess_type("file.zip")[1] is None; _ledger.append(1)

# Unknown extension returns (None, None)
assert mimetypes.guess_type("file.unknown_xyz")[0] is None; _ledger.append(1)

# Bare filename (no extension) returns (None, None)
assert mimetypes.guess_type("noextension")[0] is None; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_mimetypes_extension_compression_ops {sum(_ledger)} asserts")
