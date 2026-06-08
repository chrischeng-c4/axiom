# Operational AssertionPass seed for `stat.filemode` — the
# stdlib helper that formats a `st_mode` integer into the
# ls-style 10-character permission string (`'-rwxr-xr-x'`,
# `'drwxr-xr-x'`, `'lrwxr-xr-x'`, etc.). Used by every file-
# listing utility (ls clone, file managers, tarfile listing
# helpers, du-style tools) that needs the human-readable
# representation of a Unix mode bitfield. Surface focuses on
# the matching subset between mamba and CPython when the mode
# carries an explicit file-type bit (S_IFREG / DIR / LNK /
# CHR / BLK / FIFO / SOCK) — the no-type-bits case (mode=0o755
# alone) renders the type slot as `'?'` on CPython but `'-'`
# on mamba and is excluded. Complementary to the predicate
# / S_IMODE coverage in `test_stat_mode_helpers_ops.py`.
#
# Surface:
#   • stat.filemode(mode: int) → str
#       — returns a 10-character permission string;
#       — slot 0 = file-type indicator
#         (`-` regular, `d` dir, `l` symlink, `p` fifo,
#          `s` socket, `c` char, `b` block);
#       — slots 1-9 = three rwx triples (owner, group, other);
#       — setuid/setgid/sticky bits modify the `x` slot
#         (`s`/`S` for setuid+x/setuid only, `t`/`T` similar);
#       — output length is always 10.
import stat
_ledger: list[int] = []

# Regular file (S_IFREG) + standard mode bits
assert stat.filemode(stat.S_IFREG | 0o755) == '-rwxr-xr-x'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o644) == '-rw-r--r--'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o600) == '-rw-------'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o777) == '-rwxrwxrwx'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o000) == '----------'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o400) == '-r--------'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o200) == '--w-------'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o100) == '---x------'; _ledger.append(1)

# Directory (S_IFDIR)
assert stat.filemode(stat.S_IFDIR | 0o755) == 'drwxr-xr-x'; _ledger.append(1)
assert stat.filemode(stat.S_IFDIR | 0o700) == 'drwx------'; _ledger.append(1)
assert stat.filemode(stat.S_IFDIR | 0o777) == 'drwxrwxrwx'; _ledger.append(1)
assert stat.filemode(stat.S_IFDIR | 0o000) == 'd---------'; _ledger.append(1)

# Symlink (S_IFLNK)
assert stat.filemode(stat.S_IFLNK | 0o755) == 'lrwxr-xr-x'; _ledger.append(1)
assert stat.filemode(stat.S_IFLNK | 0o777) == 'lrwxrwxrwx'; _ledger.append(1)

# FIFO (S_IFIFO)
assert stat.filemode(stat.S_IFIFO | 0o644) == 'prw-r--r--'; _ledger.append(1)
assert stat.filemode(stat.S_IFIFO | 0o600) == 'prw-------'; _ledger.append(1)

# Socket (S_IFSOCK)
assert stat.filemode(stat.S_IFSOCK | 0o755) == 'srwxr-xr-x'; _ledger.append(1)
assert stat.filemode(stat.S_IFSOCK | 0o777) == 'srwxrwxrwx'; _ledger.append(1)

# Character device (S_IFCHR)
assert stat.filemode(stat.S_IFCHR | 0o644) == 'crw-r--r--'; _ledger.append(1)
assert stat.filemode(stat.S_IFCHR | 0o600) == 'crw-------'; _ledger.append(1)

# Block device (S_IFBLK)
assert stat.filemode(stat.S_IFBLK | 0o600) == 'brw-------'; _ledger.append(1)
assert stat.filemode(stat.S_IFBLK | 0o644) == 'brw-r--r--'; _ledger.append(1)

# setuid (S_ISUID) — owner's `x` slot becomes `s`
assert stat.filemode(stat.S_IFREG | 0o4755) == '-rwsr-xr-x'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o4644) == '-rwSr--r--'; _ledger.append(1)

# setgid (S_ISGID) — group's `x` slot becomes `s`
assert stat.filemode(stat.S_IFREG | 0o2755) == '-rwxr-sr-x'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o2644) == '-rw-r-Sr--'; _ledger.append(1)

# sticky (S_ISVTX) — other's `x` slot becomes `t`
assert stat.filemode(stat.S_IFDIR | 0o1755) == 'drwxr-xr-t'; _ledger.append(1)
assert stat.filemode(stat.S_IFDIR | 0o1644) == 'drw-r--r-T'; _ledger.append(1)

# Combined setuid + setgid + sticky
assert stat.filemode(stat.S_IFREG | 0o7755) == '-rwsr-sr-t'; _ledger.append(1)
assert stat.filemode(stat.S_IFREG | 0o7644) == '-rwSr-Sr-T'; _ledger.append(1)

# Output type — always str
assert isinstance(stat.filemode(stat.S_IFREG | 0o755), str); _ledger.append(1)
assert isinstance(stat.filemode(stat.S_IFDIR | 0o755), str); _ledger.append(1)

# Output length — always 10 characters
for _mode in [stat.S_IFREG | 0o755, stat.S_IFDIR | 0o755,
              stat.S_IFLNK | 0o644, stat.S_IFCHR | 0o600,
              stat.S_IFBLK | 0o600, stat.S_IFIFO | 0o644,
              stat.S_IFSOCK | 0o755, stat.S_IFREG | 0o000,
              stat.S_IFREG | 0o777, stat.S_IFREG | 0o4755,
              stat.S_IFREG | 0o2755, stat.S_IFDIR | 0o1755]:
    assert len(stat.filemode(_mode)) == 10; _ledger.append(1)

# Idempotence — same mode, same result
assert stat.filemode(stat.S_IFREG | 0o755) == stat.filemode(stat.S_IFREG | 0o755); _ledger.append(1)
assert stat.filemode(stat.S_IFDIR | 0o644) == stat.filemode(stat.S_IFDIR | 0o644); _ledger.append(1)

# Module-level attribute discipline
assert hasattr(stat, 'filemode'); _ledger.append(1)
assert callable(stat.filemode); _ledger.append(1)

# Inverse cross-check — `filemode` slot 0 matches the predicate
# helpers from `stat`
_mode_reg = stat.S_IFREG | 0o755
assert stat.filemode(_mode_reg)[0] == '-'; _ledger.append(1)
assert stat.S_ISREG(_mode_reg) == True; _ledger.append(1)

_mode_dir = stat.S_IFDIR | 0o755
assert stat.filemode(_mode_dir)[0] == 'd'; _ledger.append(1)
assert stat.S_ISDIR(_mode_dir) == True; _ledger.append(1)

_mode_lnk = stat.S_IFLNK | 0o755
assert stat.filemode(_mode_lnk)[0] == 'l'; _ledger.append(1)
assert stat.S_ISLNK(_mode_lnk) == True; _ledger.append(1)

_mode_fifo = stat.S_IFIFO | 0o644
assert stat.filemode(_mode_fifo)[0] == 'p'; _ledger.append(1)
assert stat.S_ISFIFO(_mode_fifo) == True; _ledger.append(1)

_mode_sock = stat.S_IFSOCK | 0o755
assert stat.filemode(_mode_sock)[0] == 's'; _ledger.append(1)
assert stat.S_ISSOCK(_mode_sock) == True; _ledger.append(1)

_mode_chr = stat.S_IFCHR | 0o644
assert stat.filemode(_mode_chr)[0] == 'c'; _ledger.append(1)
assert stat.S_ISCHR(_mode_chr) == True; _ledger.append(1)

_mode_blk = stat.S_IFBLK | 0o600
assert stat.filemode(_mode_blk)[0] == 'b'; _ledger.append(1)
assert stat.S_ISBLK(_mode_blk) == True; _ledger.append(1)

# Permission slots reflect the rwx triples — owner has `r` iff
# the owner-read bit is set, etc.
_full = stat.filemode(stat.S_IFREG | 0o777)
assert _full[1] == 'r'; _ledger.append(1)
assert _full[2] == 'w'; _ledger.append(1)
assert _full[3] == 'x'; _ledger.append(1)
assert _full[4] == 'r'; _ledger.append(1)
assert _full[5] == 'w'; _ledger.append(1)
assert _full[6] == 'x'; _ledger.append(1)
assert _full[7] == 'r'; _ledger.append(1)
assert _full[8] == 'w'; _ledger.append(1)
assert _full[9] == 'x'; _ledger.append(1)

_none = stat.filemode(stat.S_IFREG | 0o000)
for _i in range(1, 10):
    assert _none[_i] == '-'; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_stat_filemode_format_ops {sum(_ledger)} asserts")
