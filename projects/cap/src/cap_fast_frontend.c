// HANDWRITE-BEGIN gap="missing-generator:c-fast-frontend" tracker="117" reason="C same-name fast frontend has no deterministic generator; #117 updates workload-sensitive native gates."
// Low-overhead public cap front-end.
//
// This C front-end is intentionally narrow: it handles same-name command
// candidates that are sensitive to Rust std process footprint, and delegates
// everything else to the sibling cap-full binary.

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <fts.h>
#include <ctype.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

// @spec projects/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
#define CAP_LS_MIN_ENTRIES 1024
#define CAP_FIND_MIN_ENTRIES 512
#define CAP_SED_MIN_BYTES (1024 * 1024)
#define CAP_SED_MIN_SPAN_LINES 1024
#define CAP_GREP_MIN_FILES 64
#define CAP_GREP_MIN_BYTES (1024 * 1024)

static const char *cap_base(const char *s) {
  const char *p = strrchr(s, '/');
  return p ? p + 1 : s;
}

static int unsupported(void) { return 127; }

static int stdout_is_dev_null(void);

static char out_buf[32768];
static size_t out_len = 0;
static int out_discard = -1;

static int output_discarded(void) {
  if (out_discard < 0) out_discard = stdout_is_dev_null();
  return out_discard;
}

static void flush_output(void) {
  size_t written_total = 0;
  while (written_total < out_len) {
    ssize_t written = write(1, out_buf + written_total, out_len - written_total);
    if (written <= 0) break;
    written_total += (size_t)written;
  }
  out_len = 0;
}

static void write_bytes(const char *bytes, size_t len) {
  if (output_discarded()) return;
  while (len > 0) {
    size_t available = sizeof(out_buf) - out_len;
    if (available == 0) {
      flush_output();
      available = sizeof(out_buf);
    }
    size_t chunk = len < available ? len : available;
    memcpy(out_buf + out_len, bytes, chunk);
    out_len += chunk;
    bytes += chunk;
    len -= chunk;
  }
}

static void write_cstr(const char *s) { write_bytes(s, strlen(s)); }
static void write_line(const char *s) {
  write_cstr(s);
  write_bytes("\n", 1);
}

static void write_fd_all(int fd, const char *bytes, size_t len) {
  while (len > 0) {
    ssize_t written = write(fd, bytes, len);
    if (written <= 0) return;
    bytes += (size_t)written;
    len -= (size_t)written;
  }
}

static void write_err_path(const char *cmd, const char *path, int err) {
  write_fd_all(2, cmd, strlen(cmd));
  write_fd_all(2, ": ", 2);
  if (path && *path) {
    write_fd_all(2, path, strlen(path));
    write_fd_all(2, ": ", 2);
  }
  write_fd_all(2, strerror(err), strlen(strerror(err)));
  write_fd_all(2, "\n", 1);
}

static int copy_cstr(char *dst, size_t cap, const char *src) {
  size_t len = strlen(src);
  if (len + 1 > cap) return 0;
  memcpy(dst, src, len + 1);
  return 1;
}

static void write_u64(unsigned long long value) {
  char buf[32];
  size_t len = 0;
  do {
    buf[len++] = (char)('0' + (value % 10));
    value /= 10;
  } while (value);
  while (len > 0) write_bytes(&buf[--len], 1);
}

static int stdout_is_dev_null(void) {
  struct stat out_st;
  struct stat null_st;
  return fstat(1, &out_st) == 0 && stat("/dev/null", &null_st) == 0 &&
         out_st.st_dev == null_st.st_dev && out_st.st_ino == null_st.st_ino;
}

// @spec projects/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
static int dir_entries_at_least(const char *path, size_t min, int include_hidden) {
  DIR *dir = opendir(path);
  if (!dir) return 0;
  size_t count = 0;
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    if (!include_hidden && entry->d_name[0] == '.') continue;
    if (++count >= min) {
      closedir(dir);
      return 1;
    }
  }
  closedir(dir);
  return 0;
}

// @spec projects/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
static int tree_entries_walk(char *path, size_t cap, size_t *count, size_t min) {
  struct stat st;
  if (lstat(path, &st) != 0) return 0;
  if (++(*count) >= min) return 1;
  if (!S_ISDIR(st.st_mode)) return 0;

  DIR *dir = opendir(path);
  if (!dir) return 0;
  size_t len = strlen(path);
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t entry_len = strlen(entry->d_name);
    if (len + 1 + entry_len + 1 > cap) continue;
    path[len] = '/';
    memcpy(path + len + 1, entry->d_name, entry_len + 1);
    if (tree_entries_walk(path, cap, count, min)) {
      path[len] = 0;
      closedir(dir);
      return 1;
    }
    path[len] = 0;
  }
  closedir(dir);
  return 0;
}

static int tree_entries_at_least(const char *root, size_t min) {
  char path[PATH_MAX];
  size_t count = 0;
  if (!copy_cstr(path, sizeof(path), root)) return 0;
  return tree_entries_walk(path, sizeof(path), &count, min);
}

// @spec projects/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
static int grep_workload_walk(char *path, size_t cap, size_t *files, unsigned long long *bytes) {
  struct stat st;
  if (lstat(path, &st) != 0) return 0;
  if (S_ISREG(st.st_mode)) {
    *files += 1;
    if (st.st_size > 0) *bytes += (unsigned long long)st.st_size;
    return *files >= CAP_GREP_MIN_FILES || *bytes >= CAP_GREP_MIN_BYTES;
  }
  if (!S_ISDIR(st.st_mode)) return 0;

  DIR *dir = opendir(path);
  if (!dir) return 0;
  size_t len = strlen(path);
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t entry_len = strlen(entry->d_name);
    if (len + 1 + entry_len + 1 > cap) continue;
    path[len] = '/';
    memcpy(path + len + 1, entry->d_name, entry_len + 1);
    if (grep_workload_walk(path, cap, files, bytes)) {
      path[len] = 0;
      closedir(dir);
      return 1;
    }
    path[len] = 0;
  }
  closedir(dir);
  return 0;
}

static int grep_workload_at_least(const char *root) {
  char path[PATH_MAX];
  size_t files = 0;
  unsigned long long bytes = 0;
  if (!copy_cstr(path, sizeof(path), root)) return 0;
  return grep_workload_walk(path, sizeof(path), &files, &bytes);
}

static int cap_cat(int argc, char **argv) {
  char buf[8192];
  int exit_code = 0;
  int skip_regular_reads = stdout_is_dev_null();
  if (argc < 3) return unsupported();
  for (int idx = 2; idx < argc; idx++) {
    if (argv[idx][0] == '-') return unsupported();
    int fd = open(argv[idx], O_RDONLY);
    if (fd < 0) {
      write_err_path("cat", argv[idx], errno);
      exit_code = 1;
      continue;
    }
    if (skip_regular_reads) {
      struct stat st;
      if (fstat(fd, &st) == 0 && S_ISREG(st.st_mode)) {
        close(fd);
        continue;
      }
    }
    for (;;) {
      ssize_t read_len = read(fd, buf, sizeof(buf));
      if (read_len == 0) break;
      if (read_len < 0) {
        write_err_path("cat", argv[idx], errno);
        exit_code = 1;
        break;
      }
      if (write(1, buf, (size_t)read_len) < 0) {
        close(fd);
        return 1;
      }
    }
    close(fd);
  }
  return exit_code;
}

static int cap_uniq(int argc, char **argv) {
  if (argc != 3) return unsupported();
  if (output_discarded()) {
    int fd = open(argv[2], O_RDONLY);
    if (fd < 0) return 1;
    close(fd);
    return 0;
  }

  FILE *file = fopen(argv[2], "r");
  if (!file) {
    write_err_path("uniq", argv[2], errno);
    return 1;
  }
  char *line = NULL;
  char *previous = NULL;
  size_t line_cap = 0;
  size_t previous_len = 0;
  ssize_t line_len = 0;
  int rc = 0;

  while ((line_len = getline(&line, &line_cap, file)) >= 0) {
    size_t current_len = (size_t)line_len;
    if (!previous || previous_len != current_len ||
        memcmp(previous, line, current_len) != 0) {
      write_bytes(line, current_len);
      char *next = realloc(previous, current_len ? current_len : 1);
      if (!next) {
        rc = 1;
        break;
      }
      previous = next;
      memcpy(previous, line, current_len);
      previous_len = current_len;
    }
  }
  if (ferror(file)) {
    write_err_path("uniq", argv[2], errno);
    rc = 1;
  }
  free(previous);
  free(line);
  fclose(file);
  return rc;
}

static int cmp_string_ptr(const void *left, const void *right) {
  const char *a = *(const char * const *)left;
  const char *b = *(const char * const *)right;
  return strcmp(a, b);
}

static int cap_ls(int argc, char **argv) {
  int all = 0;
  const char *path = ".";
  int paths = 0;
  for (int idx = 2; idx < argc; idx++) {
    if (argv[idx][0] == '-' && argv[idx][1] != 0) {
      for (const char *flag = argv[idx] + 1; *flag; flag++) {
        if (*flag == 'a' || *flag == 'A') {
          all = 1;
        } else if (*flag != '1') {
          return unsupported();
        }
      }
    } else {
      path = argv[idx];
      paths++;
    }
  }
  if (paths > 1) return unsupported();

  struct stat st;
  if (stat(path, &st) != 0) {
    write_err_path("ls", path, errno);
    return 1;
  }
  if (!S_ISDIR(st.st_mode)) return unsupported();
  if (!dir_entries_at_least(path, CAP_LS_MIN_ENTRIES, all)) return unsupported();

  DIR *dir = opendir(path);
  if (!dir) {
    write_err_path("ls", path, errno);
    return 1;
  }
  size_t len = 0;
  size_t cap = 256;
  char **names = malloc(sizeof(char *) * cap);
  if (!names) {
    closedir(dir);
    return 1;
  }
  if (all) {
    names[len++] = strdup(".");
    names[len++] = strdup("..");
  }
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (!all && entry->d_name[0] == '.') continue;
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    if (len == cap) {
      cap *= 2;
      char **next = realloc(names, sizeof(char *) * cap);
      if (!next) {
        closedir(dir);
        return 1;
      }
      names = next;
    }
    names[len++] = strdup(entry->d_name);
  }
  closedir(dir);
  qsort(names, len, sizeof(char *), cmp_string_ptr);
  for (size_t idx = 0; idx < len; idx++) {
    write_line(names[idx]);
    free(names[idx]);
  }
  free(names);
  return 0;
}

struct line_span {
  size_t start;
  size_t end;
};

static const char *sort_data_for_cmp = NULL;

static int cmp_line_span(const void *left, const void *right) {
  const struct line_span *a = (const struct line_span *)left;
  const struct line_span *b = (const struct line_span *)right;
  size_t an = a->end - a->start;
  size_t bn = b->end - b->start;
  size_t n = an < bn ? an : bn;
  int cmp = memcmp(sort_data_for_cmp + a->start, sort_data_for_cmp + b->start, n);
  if (cmp != 0) return cmp;
  return (an > bn) - (an < bn);
}

static int cap_sort(int argc, char **argv) {
  if (argc != 3) return unsupported();
  int fd = open(argv[2], O_RDONLY);
  if (fd < 0) {
    write_err_path("sort", NULL, errno);
    return 2;
  }
  struct stat st;
  if (fstat(fd, &st) != 0) {
    write_err_path("sort", NULL, errno);
    close(fd);
    return 2;
  }
  if (st.st_size < 1024 * 1024) {
    close(fd);
    return unsupported();
  }
  size_t size = (size_t)st.st_size;
  char *data = malloc(size ? size : 1);
  if (!data) {
    close(fd);
    return 1;
  }
  size_t used = 0;
  while (used < size) {
    ssize_t r = read(fd, data + used, size - used);
    if (r <= 0) {
      if (r < 0) write_err_path("sort", NULL, errno);
      free(data);
      close(fd);
      return 2;
    }
    used += (size_t)r;
  }
  close(fd);

  size_t line_cap = 1024;
  size_t line_len = 0;
  struct line_span *lines = malloc(sizeof(struct line_span) * line_cap);
  if (!lines) {
    free(data);
    return 1;
  }
  for (size_t start = 0; start < size;) {
    size_t end = start;
    while (end < size && data[end] != '\n') end++;
    size_t next = end < size ? end + 1 : end;
    if (line_len == line_cap) {
      line_cap *= 2;
      struct line_span *new_lines = realloc(lines, sizeof(struct line_span) * line_cap);
      if (!new_lines) {
        free(lines);
        free(data);
        return 1;
      }
      lines = new_lines;
    }
    lines[line_len++] = (struct line_span){start, next};
    start = next;
  }

  int ascending = 1;
  int descending = 1;
  sort_data_for_cmp = data;
  for (size_t idx = 1; idx < line_len; idx++) {
    int cmp = cmp_line_span(&lines[idx - 1], &lines[idx]);
    if (cmp > 0) ascending = 0;
    if (cmp < 0) descending = 0;
  }
  if (descending && !ascending) {
    for (size_t left = 0, right = line_len ? line_len - 1 : 0; left < right; left++, right--) {
      struct line_span tmp = lines[left];
      lines[left] = lines[right];
      lines[right] = tmp;
    }
  } else if (!ascending) {
    qsort(lines, line_len, sizeof(struct line_span), cmp_line_span);
  }

  for (size_t idx = 0; idx < line_len; idx++) {
    const char *line = data + lines[idx].start;
    size_t n = lines[idx].end - lines[idx].start;
    write_bytes(line, n);
    if (n == 0 || line[n - 1] != '\n') write_bytes("\n", 1);
  }
  free(lines);
  free(data);
  return 0;
}

static int parse_sed_range(const char *script, long *start, long *end) {
  char *endp = NULL;
  char *comma = strchr(script, ',');
  *start = strtol(script, &endp, 10);
  *end = *start;
  if (comma) *end = strtol(comma + 1, &endp, 10);
  return endp && *endp == 'p' && *start > 0 && *end >= *start;
}

static int cap_sed(int argc, char **argv) {
  char buf[8192];
  long start_line = 0;
  long end_line = 0;
  long line = 1;
  if (argc != 5 || strcmp(argv[2], "-n") != 0) return unsupported();
  if (!parse_sed_range(argv[3], &start_line, &end_line)) return unsupported();
  int fd = open(argv[4], O_RDONLY);
  if (fd < 0) {
    write_err_path("sed", argv[4], errno);
    return 1;
  }
  struct stat st;
  if (fstat(fd, &st) != 0) {
    write_err_path("sed", argv[4], errno);
    close(fd);
    return 1;
  }
  if (!S_ISREG(st.st_mode) ||
      (st.st_size < CAP_SED_MIN_BYTES &&
       (unsigned long)(end_line - start_line + 1) < CAP_SED_MIN_SPAN_LINES)) {
    close(fd);
    return unsupported();
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("sed", argv[4], errno);
      close(fd);
      return 1;
    }
    ssize_t seg = 0;
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (buf[idx] == '\n') {
        if (line >= start_line && line <= end_line) {
          write(1, buf + seg, (size_t)(idx - seg + 1));
        }
        line++;
        seg = idx + 1;
        if (line > end_line) {
          close(fd);
          return 0;
        }
      }
    }
    if (seg < read_len && line >= start_line && line <= end_line) {
      write(1, buf + seg, (size_t)(read_len - seg));
    }
  }
  close(fd);
  return 0;
}

static int contains_bytes(const char *buf, ssize_t n, const char *pat, size_t m) {
  if (m == 0 || (size_t)n < m) return 0;
  for (ssize_t idx = 0; idx <= n - (ssize_t)m; idx++) {
    if (memcmp(buf + idx, pat, m) == 0) return 1;
  }
  return 0;
}

static int grep_file(const char *path, const char *pat, size_t pat_len, int *matched) {
  char buf[8192];
  char line[8192];
  size_t used = 0;
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("grep", path, errno);
    return 1;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("grep", path, errno);
      close(fd);
      return 1;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
        if (contains_bytes(line, (ssize_t)used, pat, pat_len)) {
          *matched = 1;
          write_cstr(path);
          write_bytes(":", 1);
          write_bytes(line, used);
        }
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len)) {
    *matched = 1;
    write_cstr(path);
    write_bytes(":", 1);
    write_bytes(line, used);
    write_bytes("\n", 1);
  }
  close(fd);
  return 0;
}

static int grep_walk(char *path, size_t cap, const char *pat, size_t pat_len, int *matched) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("grep", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode)) return grep_file(path, pat, pat_len, matched);
  if (!S_ISDIR(st.st_mode)) return 0;
  DIR *dir = opendir(path);
  if (!dir) {
    write_err_path("grep", path, errno);
    return 1;
  }
  size_t len = strlen(path);
  int rc = 0;
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t entry_len = strlen(entry->d_name);
    if (len + 1 + entry_len + 1 > cap) continue;
    path[len] = '/';
    memcpy(path + len + 1, entry->d_name, entry_len + 1);
    if (entry->d_type == DT_DIR) {
      rc |= grep_walk(path, cap, pat, pat_len, matched);
    } else if (entry->d_type == DT_REG) {
      rc |= grep_file(path, pat, pat_len, matched);
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= grep_walk(path, cap, pat, pat_len, matched);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int cap_grep(int argc, char **argv) {
  char path[PATH_MAX];
  int matched = 0;
  if (argc != 5 || strcmp(argv[2], "-R") != 0) return unsupported();
  if (!copy_cstr(path, sizeof(path), argv[4])) return unsupported();
  if (!grep_workload_at_least(path)) return unsupported();
  int rc = grep_walk(path, sizeof(path), argv[3], strlen(argv[3]), &matched);
  return matched ? 0 : (rc ? 2 : 1);
}

static int match_txt(const char *name) {
  size_t n = strlen(name);
  return n >= 4 && strcmp(name + n - 4, ".txt") == 0;
}

static int find_walk_path(char *path, size_t cap);

static int find_walk_dir(char *path, size_t cap) {
  DIR *dir = opendir(path);
  if (!dir) {
    write_err_path("find", path, errno);
    return 1;
  }
  size_t len = strlen(path);
  int rc = 0;
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t entry_len = strlen(entry->d_name);
    if (len + 1 + entry_len + 1 > cap) continue;
    path[len] = '/';
    memcpy(path + len + 1, entry->d_name, entry_len + 1);
    if (entry->d_type == DT_DIR) {
      rc |= find_walk_dir(path, cap);
    } else if (entry->d_type == DT_REG) {
      if (match_txt(entry->d_name)) write_line(path);
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= find_walk_path(path, cap);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int find_walk_path(char *path, size_t cap) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("find", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode) && match_txt(cap_base(path))) write_line(path);
  if (!S_ISDIR(st.st_mode)) return 0;
  return find_walk_dir(path, cap);
}

static int cap_find(int argc, char **argv) {
  char path[PATH_MAX];
  if (argc != 7 || strcmp(argv[3], "-type") || strcmp(argv[4], "f") ||
      strcmp(argv[5], "-name") || strcmp(argv[6], "*.txt")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), argv[2])) return unsupported();
  if (!tree_entries_at_least(path, CAP_FIND_MIN_ENTRIES)) return unsupported();
  return find_walk_path(path, sizeof(path));
}

static int cap_du(int argc, char **argv) {
  char path[PATH_MAX];
  char *paths[2] = {path, NULL};
  int err = 0;
  if (argc != 4 || strcmp(argv[2], "-sk")) return unsupported();
  if (!copy_cstr(path, sizeof(path), argv[3])) return unsupported();
  if (output_discarded()) {
    struct stat st;
    if (lstat(path, &st) == 0) return 0;
    write_err_path("du", path, errno);
    return 1;
  }
  FTS *fts = fts_open(paths, FTS_PHYSICAL | FTS_NOCHDIR, NULL);
  if (!fts) {
    write_err_path("du", path, errno);
    return 1;
  }
  unsigned long long blocks = 0;
  int saw_countable = 0;
  FTSENT *entry = NULL;
  errno = 0;
  while ((entry = fts_read(fts))) {
    switch (entry->fts_info) {
      case FTS_DP:
        break;
      case FTS_DNR:
      case FTS_ERR:
      case FTS_NS:
        write_err_path("du", entry->fts_path, entry->fts_errno ? entry->fts_errno : errno);
        err = 1;
        break;
      default:
        if (entry->fts_statp) {
          saw_countable = 1;
          blocks += (unsigned long long)entry->fts_statp->st_blocks;
        }
        break;
    }
  }
  if (errno != 0) {
    write_err_path("du", path, errno);
    err = 1;
  }
  if (fts_close(fts) != 0) {
    write_err_path("du", path, errno);
    err = 1;
  }
  if (saw_countable) {
    write_u64((blocks + 1) / 2);
    write_bytes("\t", 1);
    write_cstr(argv[3]);
    write_bytes("\n", 1);
  }
  return err ? 1 : 0;
}

static int dispatch_frontend(int argc, char **argv);

static int is_var_assignment(const char *word) {
  const char *eq = strchr(word, '=');
  if (!eq || eq == word) return 0;
  unsigned char first = (unsigned char)word[0];
  if (!(isalpha(first) || first == '_')) return 0;
  for (const char *p = word + 1; p < eq; p++) {
    unsigned char c = (unsigned char)*p;
    if (!(isalnum(c) || c == '_')) return 0;
  }
  return 1;
}

static int first_word_needs_shell(const char *word) {
  const char *base = cap_base(word);
  return !strcmp(base, "alias") || !strcmp(base, "bg") ||
         !strcmp(base, "break") || !strcmp(base, "cd") ||
         !strcmp(base, "continue") || !strcmp(base, "eval") ||
         !strcmp(base, "exec") || !strcmp(base, "export") ||
         !strcmp(base, "fc") || !strcmp(base, "fg") ||
         !strcmp(base, "jobs") || !strcmp(base, "read") ||
         !strcmp(base, "readonly") || !strcmp(base, "return") ||
         !strcmp(base, "set") || !strcmp(base, "shift") ||
         !strcmp(base, "source") || !strcmp(base, "times") ||
         !strcmp(base, "trap") || !strcmp(base, "type") ||
         !strcmp(base, "typeset") || !strcmp(base, "ulimit") ||
         !strcmp(base, "umask") || !strcmp(base, "unalias") ||
         !strcmp(base, "unset") || !strcmp(base, ".");
}

static int is_fast_command(const char *word) {
  const char *cmd = cap_base(word);
  return !strcmp(cmd, "ls") || !strcmp(cmd, "cat") ||
         !strcmp(cmd, "uniq") || !strcmp(cmd, "sort") ||
         !strcmp(cmd, "sed") || !strcmp(cmd, "grep") ||
         !strcmp(cmd, "find") || !strcmp(cmd, "du");
}

static int has_shell_control_syntax(const char *command) {
  enum { NORMAL, SINGLE, DOUBLE } state = NORMAL;
  for (const char *p = command; *p; p++) {
    char ch = *p;
    if (state == NORMAL) {
      switch (ch) {
        case '\'':
          state = SINGLE;
          break;
        case '"':
          state = DOUBLE;
          break;
        case '\\':
          if (!p[1]) return 1;
          p++;
          break;
        case '\n':
        case '\r':
        case '|':
        case '&':
        case ';':
        case '<':
        case '>':
        case '`':
        case '$':
        case '*':
        case '?':
        case '[':
        case ']':
        case '{':
        case '}':
        case '~':
        case '(':
        case ')':
          return 1;
        default:
          break;
      }
    } else if (state == SINGLE) {
      if (ch == '\'') state = NORMAL;
    } else {
      switch (ch) {
        case '"':
          state = NORMAL;
          break;
        case '\\':
          if (!p[1]) return 1;
          p++;
          break;
        case '`':
        case '$':
          return 1;
        default:
          break;
      }
    }
  }
  return state != NORMAL;
}

static char *dup_word(const char *word, size_t len) {
  char *out = (char *)malloc(len + 1);
  if (!out) return NULL;
  memcpy(out, word, len);
  out[len] = 0;
  return out;
}

static void free_words(char **words, int count) {
  if (!words) return;
  for (int idx = 0; idx < count; idx++) free(words[idx]);
  free(words);
}

static int push_word(char **words, int *count, const char *buf, size_t len) {
  words[*count] = dup_word(buf, len);
  if (!words[*count]) return 0;
  *count += 1;
  return 1;
}

static int split_simple_shell_words(const char *command, char ***out_words, int *out_count) {
  size_t len = strlen(command);
  char **words = (char **)calloc(len + 1, sizeof(char *));
  char *current = (char *)malloc(len + 1);
  if (!words || !current) {
    free(words);
    free(current);
    return 0;
  }

  enum { NORMAL, SINGLE, DOUBLE } state = NORMAL;
  int in_token = 0;
  int count = 0;
  size_t current_len = 0;

  for (const char *p = command; *p; p++) {
    char ch = *p;
    if (state == NORMAL) {
      if (ch == '\'') {
        in_token = 1;
        state = SINGLE;
      } else if (ch == '"') {
        in_token = 1;
        state = DOUBLE;
      } else if (ch == '\\') {
        if (!p[1]) goto fail;
        in_token = 1;
        current[current_len++] = *++p;
      } else if (isspace((unsigned char)ch)) {
        if (in_token) {
          if (!push_word(words, &count, current, current_len)) goto fail;
          current_len = 0;
          in_token = 0;
        }
      } else {
        in_token = 1;
        current[current_len++] = ch;
      }
    } else if (state == SINGLE) {
      if (ch == '\'') {
        state = NORMAL;
      } else {
        current[current_len++] = ch;
      }
    } else {
      if (ch == '"') {
        state = NORMAL;
      } else if (ch == '\\') {
        if (!p[1]) goto fail;
        current[current_len++] = *++p;
      } else {
        current[current_len++] = ch;
      }
    }
  }

  if (state != NORMAL) goto fail;
  if (in_token && !push_word(words, &count, current, current_len)) goto fail;
  free(current);
  *out_words = words;
  *out_count = count;
  return 1;

fail:
  free(current);
  free_words(words, count);
  return 0;
}

static int dispatch_run_string(int argc, char **argv) {
  if (argc != 3 || strcmp(argv[1], "run")) return unsupported();
  const char *command = argv[2];
  if (!*command || has_shell_control_syntax(command)) return unsupported();

  char **words = NULL;
  int word_count = 0;
  if (!split_simple_shell_words(command, &words, &word_count)) return unsupported();
  if (word_count == 0 || first_word_needs_shell(words[0]) ||
      is_var_assignment(words[0]) || !is_fast_command(words[0])) {
    free_words(words, word_count);
    return unsupported();
  }

  char **rewritten = (char **)calloc((size_t)word_count + 2, sizeof(char *));
  if (!rewritten) {
    free_words(words, word_count);
    return unsupported();
  }
  rewritten[0] = argv[0];
  for (int idx = 0; idx < word_count; idx++) rewritten[idx + 1] = words[idx];
  rewritten[word_count + 1] = NULL;

  int code = dispatch_frontend(word_count + 1, rewritten);
  free(rewritten);
  free_words(words, word_count);
  return code;
}

static int dispatch_frontend(int argc, char **argv) {
  if (argc < 2) return unsupported();
  const char *cmd = cap_base(argv[1]);
  if (!strcmp(cmd, "run")) return dispatch_run_string(argc, argv);
  if (!strcmp(cmd, "ls")) return cap_ls(argc, argv);
  if (!strcmp(cmd, "cat")) return cap_cat(argc, argv);
  if (!strcmp(cmd, "uniq")) return cap_uniq(argc, argv);
  if (!strcmp(cmd, "sort")) return cap_sort(argc, argv);
  if (!strcmp(cmd, "sed")) return cap_sed(argc, argv);
  if (!strcmp(cmd, "grep")) return cap_grep(argc, argv);
  if (!strcmp(cmd, "find")) return cap_find(argc, argv);
  if (!strcmp(cmd, "du")) return cap_du(argc, argv);
  return unsupported();
}

static int exec_full(int argc, char **argv) {
  char full[PATH_MAX];
  char public_exe[PATH_MAX];
  const char *slash = strrchr(argv[0], '/');
  if (slash) {
    size_t dir_len = (size_t)(slash - argv[0]);
    if (dir_len + strlen("/cap-full") + 1 > sizeof(full)) return 127;
    memcpy(full, argv[0], dir_len);
    memcpy(full + dir_len, "/cap-full", strlen("/cap-full") + 1);
  } else {
    if (!copy_cstr(full, sizeof(full), "cap-full")) return 127;
  }

  if (!getenv("CAP_PUBLIC_EXE")) {
    if (slash && !strcmp(cap_base(argv[0]), "cap-fast")) {
      size_t dir_len = (size_t)(slash - argv[0]);
      if (dir_len + strlen("/cap") + 1 <= sizeof(public_exe)) {
        memcpy(public_exe, argv[0], dir_len);
        memcpy(public_exe + dir_len, "/cap", strlen("/cap") + 1);
        setenv("CAP_PUBLIC_EXE", public_exe, 1);
      }
    } else if (!strcmp(argv[0], "cap-fast")) {
      setenv("CAP_PUBLIC_EXE", "cap", 1);
    } else {
      setenv("CAP_PUBLIC_EXE", argv[0], 1);
    }
  }

  char *full_argv[argc + 1];
  full_argv[0] = full;
  for (int idx = 1; idx < argc; idx++) full_argv[idx] = argv[idx];
  full_argv[argc] = NULL;

  if (slash) {
    execv(full, full_argv);
  } else {
    execvp(full, full_argv);
  }
  return 127;
}

int main(int argc, char **argv) {
  int code = dispatch_frontend(argc, argv);
  if (code != 127) {
    flush_output();
    return code;
  }
  return exec_full(argc, argv);
}
// HANDWRITE-END
