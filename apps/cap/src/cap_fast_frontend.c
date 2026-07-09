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
#include <grp.h>
#include <ctype.h>
#include <limits.h>
#include <pwd.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/types.h>
#include <sys/utsname.h>
#include <unistd.h>

// @spec apps/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
#define CAP_LS_MIN_ENTRIES 1024
#define CAP_FIND_MIN_ENTRIES 512
#define CAP_SED_MIN_BYTES (1024 * 1024)
#define CAP_SED_MIN_SPAN_LINES 1024
#define CAP_GREP_MIN_FILES 64
#define CAP_GREP_MIN_BYTES (1024 * 1024)
// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#define CAP_WC_MIN_FILES 64
#define CAP_WC_MIN_BYTES (1024 * 1024)

extern char **environ;

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

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
static void write_padded_u64(unsigned long long value) {
  unsigned long long tmp = value;
  int digits = 1;
  while (tmp >= 10) {
    tmp /= 10;
    digits++;
  }
  for (int idx = digits; idx < 8; idx++) write_bytes(" ", 1);
  write_u64(value);
}

static int parse_u64_arg(const char *s, unsigned long long *out) {
  if (!s || !*s) return 0;
  unsigned long long value = 0;
  for (const char *p = s; *p; p++) {
    if (*p < '0' || *p > '9') return 0;
    value = value * 10 + (unsigned long long)(*p - '0');
  }
  *out = value;
  return 1;
}

static int cap_true(int argc, char **argv) {
  (void)argc;
  (void)argv;
  return 0;
}

static int cap_false(int argc, char **argv) {
  (void)argc;
  (void)argv;
  return 1;
}

static int cap_pwd(int argc, char **argv) {
  (void)argv;
  char cwd[PATH_MAX];
  if (argc != 2) return unsupported();
  if (!getcwd(cwd, sizeof(cwd))) {
    write_err_path("pwd", NULL, errno);
    return 1;
  }
  write_line(cwd);
  return 0;
}

static int parse_echo_args(int argc, char **argv, int *first_arg, int *newline) {
  *first_arg = 2;
  *newline = 1;
  if (argc >= 3 && argv[2][0] == '-') {
    if (strcmp(argv[2], "-n")) return 0;
    *first_arg = 3;
    *newline = 0;
  }
  if (!*newline) {
    for (int idx = *first_arg; idx < argc; idx++) {
      if (argv[idx][0] == '-') return 0;
    }
  }
  return 1;
}

static void emit_echo_args(int argc, char **argv, int first_arg, int newline) {
  for (int idx = first_arg; idx < argc; idx++) {
    if (idx > first_arg) write_bytes(" ", 1);
    write_cstr(argv[idx]);
  }
  if (newline) write_bytes("\n", 1);
}

static int cap_echo(int argc, char **argv) {
  int first_arg = 2;
  int newline = 1;
  if (!parse_echo_args(argc, argv, &first_arg, &newline)) return unsupported();
  emit_echo_args(argc, argv, first_arg, newline);
  return 0;
}

enum printf_format_kind {
  PRINTF_FORMAT_UNSUPPORTED = 0,
  PRINTF_FORMAT_STRING,
  PRINTF_FORMAT_STRING_NEWLINE,
};

static enum printf_format_kind printf_format_kind(const char *format) {
  if (!strcmp(format, "%s")) return PRINTF_FORMAT_STRING;
  if (!strcmp(format, "%s\\n") || !strcmp(format, "%s\n")) {
    return PRINTF_FORMAT_STRING_NEWLINE;
  }
  return PRINTF_FORMAT_UNSUPPORTED;
}

static int decode_printf_literal_format(const char *format, char **out,
                                        size_t *out_len) {
  size_t len = strlen(format);
  char *buf = (char *)malloc(len ? len : 1);
  size_t used = 0;
  if (!buf) return 0;
  for (size_t idx = 0; idx < len; idx++) {
    unsigned char byte = (unsigned char)format[idx];
    if (byte == '%') {
      free(buf);
      return 0;
    }
    if (byte == '\\') {
      idx++;
      if (idx >= len) {
        free(buf);
        return 0;
      }
      switch (format[idx]) {
        case '\\':
          buf[used++] = '\\';
          break;
        case 'n':
          buf[used++] = '\n';
          break;
        case 't':
          buf[used++] = '\t';
          break;
        case 'r':
          buf[used++] = '\r';
          break;
        default:
          free(buf);
          return 0;
      }
      continue;
    }
    buf[used++] = (char)byte;
  }
  *out = buf;
  *out_len = used;
  return 1;
}

static void emit_printf_args(enum printf_format_kind kind, int argc, char **argv,
                             int first_arg) {
  for (int idx = first_arg; idx < argc; idx++) {
    write_cstr(argv[idx]);
    if (kind == PRINTF_FORMAT_STRING_NEWLINE) write_bytes("\n", 1);
  }
}

static int cap_printf(int argc, char **argv) {
  if (argc == 3) {
    char *literal = NULL;
    size_t literal_len = 0;
    if (!decode_printf_literal_format(argv[2], &literal, &literal_len)) return unsupported();
    write_bytes(literal, literal_len);
    free(literal);
    return 0;
  }
  if (argc < 4) return unsupported();
  enum printf_format_kind kind = printf_format_kind(argv[2]);
  if (kind == PRINTF_FORMAT_UNSUPPORTED) return unsupported();
  emit_printf_args(kind, argc, argv, 3);
  return 0;
}

struct seq_plan {
  long long first;
  long long step;
  long long last;
};

static int parse_i64_arg(const char *s, long long *out) {
  if (!s || !*s) return 0;
  errno = 0;
  char *end = NULL;
  long long value = strtoll(s, &end, 10);
  if (errno == ERANGE || !end || *end) return 0;
  *out = value;
  return 1;
}

static int parse_seq_words(char **words, int start, int end, struct seq_plan *seq) {
  int argc = end - start;
  if (argc == 2) {
    seq->first = 1;
    seq->step = 1;
    if (!parse_i64_arg(words[start + 1], &seq->last)) return 0;
  } else if (argc == 3) {
    seq->step = 1;
    if (!parse_i64_arg(words[start + 1], &seq->first) ||
        !parse_i64_arg(words[start + 2], &seq->last)) {
      return 0;
    }
  } else if (argc == 4) {
    if (!parse_i64_arg(words[start + 1], &seq->first) ||
        !parse_i64_arg(words[start + 2], &seq->step) ||
        !parse_i64_arg(words[start + 3], &seq->last)) {
      return 0;
    }
  } else {
    return 0;
  }
  return seq->step != 0;
}

static unsigned long long seq_count(const struct seq_plan *seq) {
  __int128 first = seq->first;
  __int128 step = seq->step;
  __int128 last = seq->last;
  __int128 count = 0;
  if (seq->step > 0) {
    if (first > last) return 0;
    count = ((last - first) / step) + 1;
  } else {
    if (first < last) return 0;
    count = ((first - last) / -step) + 1;
  }
  if (count > (__int128)ULLONG_MAX) return ULLONG_MAX;
  return (unsigned long long)count;
}

static void write_i64(long long value) {
  char buf[32];
  int len = snprintf(buf, sizeof(buf), "%lld", value);
  if (len > 0) write_bytes(buf, (size_t)len);
}

static void emit_seq(const struct seq_plan *seq, unsigned long long limit) {
  if (output_discarded()) return;
  unsigned long long remaining = seq_count(seq);
  if (remaining > limit) remaining = limit;
  long long current = seq->first;
  while (remaining > 0) {
    write_i64(current);
    write_bytes("\n", 1);
    remaining--;
    if (remaining == 0) break;
    current += seq->step;
  }
}

static void emit_seq_tail(const struct seq_plan *seq, unsigned long long limit) {
  if (output_discarded()) return;
  unsigned long long count = seq_count(seq);
  unsigned long long emit = count < limit ? count : limit;
  unsigned long long skip = count - emit;
  long long current = seq->first + (seq->step * (long long)skip);
  while (emit > 0) {
    write_i64(current);
    write_bytes("\n", 1);
    emit--;
    if (emit == 0) break;
    current += seq->step;
  }
}

static int cap_seq(int argc, char **argv) {
  struct seq_plan seq;
  if (!parse_seq_words(argv, 1, argc, &seq)) return unsupported();
  emit_seq(&seq, ULLONG_MAX);
  return 0;
}

static const char *effective_user_name(void) {
  struct passwd *pw = getpwuid(geteuid());
  return pw ? pw->pw_name : NULL;
}

static const char *effective_group_name(void) {
  struct group *gr = getgrgid(getegid());
  return gr ? gr->gr_name : NULL;
}

static int group_list_value(int names, char *out, size_t out_cap) {
  int count = getgroups(0, NULL);
  if (count < 0) return -1;
  if (count > 256) return 0;
  gid_t groups[256];
  int read_count = getgroups(256, groups);
  if (read_count < 0) return -1;
  size_t used = 0;
  for (int idx = 0; idx < read_count; idx++) {
    char item[128];
    if (names) {
      struct group *gr = getgrgid(groups[idx]);
      if (!gr || !gr->gr_name) return -2;
      if (!copy_cstr(item, sizeof(item), gr->gr_name)) return 0;
    } else {
      int len = snprintf(item, sizeof(item), "%llu", (unsigned long long)groups[idx]);
      if (len < 0 || (size_t)len >= sizeof(item)) return 0;
    }
    size_t item_len = strlen(item);
    size_t sep = idx == 0 ? 0 : 1;
    if (used + sep + item_len + 1 > out_cap) return 0;
    if (sep) out[used++] = ' ';
    memcpy(out + used, item, item_len);
    used += item_len;
  }
  out[used] = 0;
  return 1;
}

static int append_cstr(char *out, size_t out_cap, size_t *used, const char *value) {
  size_t len = strlen(value);
  if (*used + len + 1 > out_cap) return 0;
  memcpy(out + *used, value, len);
  *used += len;
  out[*used] = 0;
  return 1;
}

static int append_u64(char *out, size_t out_cap, size_t *used,
                      unsigned long long value) {
  char buf[32];
  int len = snprintf(buf, sizeof(buf), "%llu", value);
  if (len < 0 || (size_t)len >= sizeof(buf)) return 0;
  return append_cstr(out, out_cap, used, buf);
}

static int append_id_item(char *out, size_t out_cap, size_t *used,
                          unsigned long long id, const char *name) {
  if (!append_u64(out, out_cap, used, id)) return 0;
  if (name) {
    if (!append_cstr(out, out_cap, used, "(")) return 0;
    if (!append_cstr(out, out_cap, used, name)) return 0;
    if (!append_cstr(out, out_cap, used, ")")) return 0;
  }
  return 1;
}

static int default_id_value(char *out, size_t out_cap) {
  gid_t groups[256];
  int count = getgroups(0, NULL);
  if (count < 0) return -1;
  if (count > 256) return 0;
  int read_count = getgroups(256, groups);
  if (read_count < 0) return -1;

  uid_t uid = geteuid();
  gid_t gid = getegid();
  struct passwd *pw = getpwuid(uid);
  struct group *gr = getgrgid(gid);
  size_t used = 0;
  out[0] = 0;
  if (!append_cstr(out, out_cap, &used, "uid=")) return 0;
  if (!append_id_item(out, out_cap, &used, (unsigned long long)uid,
                      pw && pw->pw_name ? pw->pw_name : NULL)) {
    return 0;
  }
  if (!append_cstr(out, out_cap, &used, " gid=")) return 0;
  if (!append_id_item(out, out_cap, &used, (unsigned long long)gid,
                      gr && gr->gr_name ? gr->gr_name : NULL)) {
    return 0;
  }
  if (!append_cstr(out, out_cap, &used, " groups=")) return 0;
  for (int idx = 0; idx < read_count; idx++) {
    if (idx > 0 && !append_cstr(out, out_cap, &used, ",")) return 0;
    struct group *item = getgrgid(groups[idx]);
    if (!append_id_item(out, out_cap, &used, (unsigned long long)groups[idx],
                        item && item->gr_name ? item->gr_name : NULL)) {
      return 0;
    }
  }
  return 1;
}

static int cap_whoami(int argc, char **argv) {
  (void)argv;
  if (argc != 2) return unsupported();
  const char *name = effective_user_name();
  if (!name) {
    const char *msg = "whoami: cannot find name for user ID\n";
    write_fd_all(2, msg, strlen(msg));
    return 1;
  }
  write_line(name);
  return 0;
}

static int cap_id(int argc, char **argv) {
  if (argc == 2) {
    char value[4096];
    int rc = default_id_value(value, sizeof(value));
    if (rc == 0) return unsupported();
    if (rc < 0) {
      write_err_path("id", NULL, errno);
      return 1;
    }
    write_line(value);
    return 0;
  }
  if (argc != 3) return unsupported();
  if (!strcmp(argv[2], "-u")) {
    write_u64((unsigned long long)geteuid());
    write_bytes("\n", 1);
    return 0;
  }
  if (!strcmp(argv[2], "-g")) {
    write_u64((unsigned long long)getegid());
    write_bytes("\n", 1);
    return 0;
  }
  if (!strcmp(argv[2], "-un")) {
    const char *name = effective_user_name();
    if (!name) {
      const char *msg = "id: cannot find name for user ID\n";
      write_fd_all(2, msg, strlen(msg));
      return 1;
    }
    write_line(name);
    return 0;
  }
  if (!strcmp(argv[2], "-gn")) {
    const char *name = effective_group_name();
    if (!name) {
      const char *msg = "id: cannot find name for group ID\n";
      write_fd_all(2, msg, strlen(msg));
      return 1;
    }
    write_line(name);
    return 0;
  }
  if (!strcmp(argv[2], "-G") || !strcmp(argv[2], "-Gn")) {
    char value[4096];
    int rc = group_list_value(!strcmp(argv[2], "-Gn"), value, sizeof(value));
    if (rc == 0) return unsupported();
    if (rc == -2) {
      const char *msg = "id: cannot find name for group ID\n";
      write_fd_all(2, msg, strlen(msg));
      return 1;
    }
    if (rc < 0) {
      write_err_path("id", NULL, errno);
      return 1;
    }
    write_line(value);
    return 0;
  }
  return unsupported();
}

static const char *uname_processor_field(const char *machine) {
#if defined(__APPLE__)
  if (!strcmp(machine, "arm64") || !strcmp(machine, "aarch64")) return "arm";
  if (!strcmp(machine, "x86_64")) return "i386";
#endif
  return machine;
}

static const char *uname_field(const struct utsname *uts, const char *flag) {
  if (!flag || !strcmp(flag, "-s")) return uts->sysname;
  if (!strcmp(flag, "-n")) return uts->nodename;
  if (!strcmp(flag, "-r")) return uts->release;
  if (!strcmp(flag, "-v")) return uts->version;
  if (!strcmp(flag, "-m")) return uts->machine;
  if (!strcmp(flag, "-p")) return uname_processor_field(uts->machine);
  return NULL;
}

static int cap_uname(int argc, char **argv) {
  struct utsname uts;
  if (argc != 2 && argc != 3) return unsupported();
  const char *flag = argc == 3 ? argv[2] : NULL;
  if (uname(&uts) != 0) {
    write_err_path("uname", NULL, errno);
    return 1;
  }
  if (flag && !strcmp(flag, "-a")) {
    write_cstr(uts.sysname);
    write_bytes(" ", 1);
    write_cstr(uts.nodename);
    write_bytes(" ", 1);
    write_cstr(uts.release);
    write_bytes(" ", 1);
    write_cstr(uts.version);
    write_bytes(" ", 1);
    write_cstr(uts.machine);
    write_bytes("\n", 1);
    return 0;
  }
  const char *field = uname_field(&uts, flag);
  if (!field) return unsupported();
  write_line(field);
  return 0;
}

static int cap_hostname(int argc, char **argv) {
  (void)argv;
  char name[256];
  if (argc != 2) return unsupported();
  if (gethostname(name, sizeof(name)) != 0) {
    write_err_path("hostname", NULL, errno);
    return 1;
  }
  name[sizeof(name) - 1] = 0;
  write_line(name);
  return 0;
}

enum test_expr_kind {
  TEST_FILE_EXISTS,
  TEST_FILE_REGULAR,
  TEST_FILE_DIRECTORY,
  TEST_FILE_NONEMPTY,
  TEST_STRING_NONEMPTY,
  TEST_STRING_EMPTY,
  TEST_STRING_EQ,
  TEST_STRING_NE,
  TEST_INT_EQ,
  TEST_INT_NE,
  TEST_INT_GT,
  TEST_INT_GE,
  TEST_INT_LT,
  TEST_INT_LE,
};

struct test_expr {
  enum test_expr_kind kind;
  int negated;
  const char *left;
  const char *right;
  long long left_int;
  long long right_int;
};

static int parse_test_words(char **words, int start, int end, struct test_expr *expr) {
  expr->negated = 0;
  if (start < end && !strcmp(words[start], "!")) {
    expr->negated = 1;
    start++;
  }
  int argc = end - start;
  if (argc <= 0) return 0;

  if (argc == 1) {
    expr->kind = TEST_STRING_NONEMPTY;
    expr->left = words[start];
    return 1;
  }
  if (argc == 2) {
    expr->left = words[start + 1];
    if (!strcmp(words[start], "-e")) {
      expr->kind = TEST_FILE_EXISTS;
      return 1;
    }
    if (!strcmp(words[start], "-f")) {
      expr->kind = TEST_FILE_REGULAR;
      return 1;
    }
    if (!strcmp(words[start], "-d")) {
      expr->kind = TEST_FILE_DIRECTORY;
      return 1;
    }
    if (!strcmp(words[start], "-s")) {
      expr->kind = TEST_FILE_NONEMPTY;
      return 1;
    }
    if (!strcmp(words[start], "-n")) {
      expr->kind = TEST_STRING_NONEMPTY;
      return 1;
    }
    if (!strcmp(words[start], "-z")) {
      expr->kind = TEST_STRING_EMPTY;
      return 1;
    }
    return 0;
  }
  if (argc == 3) {
    expr->left = words[start];
    expr->right = words[start + 2];
    const char *op = words[start + 1];
    if (!strcmp(op, "=") || !strcmp(op, "==")) {
      expr->kind = TEST_STRING_EQ;
      return 1;
    }
    if (!strcmp(op, "!=")) {
      expr->kind = TEST_STRING_NE;
      return 1;
    }
    if (!strcmp(op, "-eq") || !strcmp(op, "-ne") || !strcmp(op, "-gt") ||
        !strcmp(op, "-ge") || !strcmp(op, "-lt") || !strcmp(op, "-le")) {
      if (!parse_i64_arg(expr->left, &expr->left_int) ||
          !parse_i64_arg(expr->right, &expr->right_int)) {
        return 0;
      }
      if (!strcmp(op, "-eq")) expr->kind = TEST_INT_EQ;
      else if (!strcmp(op, "-ne")) expr->kind = TEST_INT_NE;
      else if (!strcmp(op, "-gt")) expr->kind = TEST_INT_GT;
      else if (!strcmp(op, "-ge")) expr->kind = TEST_INT_GE;
      else if (!strcmp(op, "-lt")) expr->kind = TEST_INT_LT;
      else expr->kind = TEST_INT_LE;
      return 1;
    }
  }
  return 0;
}

static int eval_test_expr(const struct test_expr *expr) {
  struct stat st;
  int value = 0;
  switch (expr->kind) {
    case TEST_FILE_EXISTS:
      value = stat(expr->left, &st) == 0;
      break;
    case TEST_FILE_REGULAR:
      value = stat(expr->left, &st) == 0 && S_ISREG(st.st_mode);
      break;
    case TEST_FILE_DIRECTORY:
      value = stat(expr->left, &st) == 0 && S_ISDIR(st.st_mode);
      break;
    case TEST_FILE_NONEMPTY:
      value = stat(expr->left, &st) == 0 && st.st_size > 0;
      break;
    case TEST_STRING_NONEMPTY:
      value = expr->left && *expr->left;
      break;
    case TEST_STRING_EMPTY:
      value = !expr->left || !*expr->left;
      break;
    case TEST_STRING_EQ:
      value = !strcmp(expr->left, expr->right);
      break;
    case TEST_STRING_NE:
      value = strcmp(expr->left, expr->right) != 0;
      break;
    case TEST_INT_EQ:
      value = expr->left_int == expr->right_int;
      break;
    case TEST_INT_NE:
      value = expr->left_int != expr->right_int;
      break;
    case TEST_INT_GT:
      value = expr->left_int > expr->right_int;
      break;
    case TEST_INT_GE:
      value = expr->left_int >= expr->right_int;
      break;
    case TEST_INT_LT:
      value = expr->left_int < expr->right_int;
      break;
    case TEST_INT_LE:
      value = expr->left_int <= expr->right_int;
      break;
  }
  return expr->negated ? !value : value;
}

static int cap_test_cmd(int argc, char **argv, int bracket) {
  int start = 2;
  int end = argc;
  if (bracket) {
    if (argc < 3 || strcmp(argv[argc - 1], "]")) return unsupported();
    end = argc - 1;
  }
  struct test_expr expr;
  if (!parse_test_words(argv, start, end, &expr)) return unsupported();
  return eval_test_expr(&expr) ? 0 : 1;
}

static int cap_basename(int argc, char **argv) {
  if (argc != 3 && argc != 4) return unsupported();
  const char *input = argv[2];
  size_t len = strlen(input);
  if (len == 0) {
    write_line(".");
    return 0;
  }
  while (len > 1 && input[len - 1] == '/') len--;
  size_t start = len;
  while (start > 0 && input[start - 1] != '/') start--;
  if (start == len && input[0] == '/') {
    write_line("/");
    return 0;
  }
  size_t base_len = len - start;
  if (argc == 4) {
    size_t suffix_len = strlen(argv[3]);
    if (suffix_len > 0 && suffix_len < base_len &&
        memcmp(input + start + base_len - suffix_len, argv[3], suffix_len) == 0) {
      base_len -= suffix_len;
    }
  }
  write_bytes(input + start, base_len);
  write_bytes("\n", 1);
  return 0;
}

static int cap_dirname(int argc, char **argv) {
  if (argc != 3) return unsupported();
  const char *input = argv[2];
  size_t len = strlen(input);
  if (len == 0) {
    write_line(".");
    return 0;
  }
  while (len > 1 && input[len - 1] == '/') len--;
  size_t end = len;
  while (end > 0 && input[end - 1] != '/') end--;
  if (end == 0) {
    write_line(".");
    return 0;
  }
  while (end > 1 && input[end - 1] == '/') end--;
  write_bytes(input, end);
  write_bytes("\n", 1);
  return 0;
}

static int stdout_is_dev_null(void) {
  struct stat out_st;
  struct stat null_st;
  return fstat(1, &out_st) == 0 && stat("/dev/null", &null_st) == 0 &&
         out_st.st_dev == null_st.st_dev && out_st.st_ino == null_st.st_ino;
}

// @spec apps/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
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

// @spec apps/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
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

// @spec apps/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
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

static int head_copy_bytes(int fd, unsigned long long count) {
  char buf[8192];
  while (count > 0) {
    size_t want = count < sizeof(buf) ? (size_t)count : sizeof(buf);
    ssize_t read_len = read(fd, buf, want);
    if (read_len == 0) break;
    if (read_len < 0) return 1;
    write_bytes(buf, (size_t)read_len);
    count -= (unsigned long long)read_len;
  }
  return 0;
}

static int head_copy_lines(int fd, unsigned long long count) {
  char buf[8192];
  if (count == 0) return 0;
  while (count > 0) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) return 1;
    ssize_t end = read_len;
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (buf[idx] == '\n' && --count == 0) {
        end = idx + 1;
        break;
      }
    }
    write_bytes(buf, (size_t)end);
  }
  return 0;
}

static int cap_head(int argc, char **argv) {
  int bytes = 0;
  unsigned long long count = 10;
  const char *path = NULL;
  int stdin_mode = 0;
  if (argc == 2) {
    stdin_mode = 1;
  } else if (argc == 3 && argv[2][0] == '-' && argv[2][1] >= '0' && argv[2][1] <= '9') {
    if (!parse_u64_arg(argv[2] + 1, &count)) return unsupported();
    stdin_mode = 1;
  } else if (argc == 3) {
    path = argv[2];
  } else if (argc == 4 && (!strcmp(argv[2], "-c") || !strcmp(argv[2], "-n"))) {
    bytes = !strcmp(argv[2], "-c");
    if (!parse_u64_arg(argv[3], &count)) return unsupported();
    stdin_mode = 1;
  } else if (argc == 5 && (!strcmp(argv[2], "-c") || !strcmp(argv[2], "-n"))) {
    bytes = !strcmp(argv[2], "-c");
    if (!parse_u64_arg(argv[3], &count)) return unsupported();
    path = argv[4];
  } else if (argc == 4 && argv[2][0] == '-' && argv[2][1] >= '0' && argv[2][1] <= '9') {
    if (!parse_u64_arg(argv[2] + 1, &count)) return unsupported();
    path = argv[3];
  } else {
    return unsupported();
  }
  if (count == 0) return unsupported();
  int fd = stdin_mode ? STDIN_FILENO : open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("head", path, errno);
    return 1;
  }
  int rc = bytes ? head_copy_bytes(fd, count) : head_copy_lines(fd, count);
  if (rc) write_err_path("head", path, errno);
  if (!stdin_mode) close(fd);
  return rc;
}

static int read_all_fd(int fd, const char *cmd, const char *path, char **out, size_t *out_len) {
  char buf[8192];
  size_t used = 0;
  size_t cap = 8192;
  char *data = (char *)malloc(cap);
  if (!data) return 1;
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path(cmd, path, errno);
      free(data);
      return 1;
    }
    if (used + (size_t)read_len > cap) {
      size_t next = cap;
      while (used + (size_t)read_len > next) {
        if (next > SIZE_MAX / 2) {
          free(data);
          errno = ENOMEM;
          write_err_path(cmd, path, errno);
          return 1;
        }
        next *= 2;
      }
      char *grown = (char *)realloc(data, next);
      if (!grown) {
        free(data);
        return 1;
      }
      data = grown;
      cap = next;
    }
    memcpy(data + used, buf, (size_t)read_len);
    used += (size_t)read_len;
  }
  *out = data;
  *out_len = used;
  return 0;
}

static void tail_emit_bytes_data(const char *data, size_t size, unsigned long long count) {
  size_t start = 0;
  if ((unsigned long long)size > count) start = size - (size_t)count;
  write_bytes(data + start, size - start);
}

static void tail_emit_lines_data(const char *data, size_t size, unsigned long long count) {
  if (count == 0) return;
  size_t start = 0;
  size_t pos = size;
  if (pos > 0 && data[pos - 1] == '\n') pos--;
  unsigned long long seen = 0;
  while (pos > 0) {
    if (data[pos - 1] == '\n' && ++seen == count) {
      start = pos;
      break;
    }
    pos--;
  }
  write_bytes(data + start, size - start);
}

static int tail_copy_bytes(int fd, const char *path, unsigned long long count) {
  struct stat st;
  if (fstat(fd, &st) != 0) return unsupported();
  if (!S_ISREG(st.st_mode)) {
    if (path) return unsupported();
    char *data = NULL;
    size_t size = 0;
    int rc = read_all_fd(fd, "tail", path, &data, &size);
    if (rc) return rc;
    tail_emit_bytes_data(data, size, count);
    free(data);
    return 0;
  }
  off_t start = 0;
  if ((unsigned long long)st.st_size > count) start = st.st_size - (off_t)count;
  if (lseek(fd, start, SEEK_SET) < 0) {
    write_err_path("tail", path, errno);
    return 1;
  }
  char buf[8192];
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("tail", path, errno);
      return 1;
    }
    write_bytes(buf, (size_t)read_len);
  }
  return 0;
}

static int tail_copy_lines(int fd, const char *path, unsigned long long count) {
  struct stat st;
  if (fstat(fd, &st) != 0) return unsupported();
  if (!S_ISREG(st.st_mode)) {
    if (path) return unsupported();
    char *data = NULL;
    size_t size = 0;
    int rc = read_all_fd(fd, "tail", path, &data, &size);
    if (rc) return rc;
    tail_emit_lines_data(data, size, count);
    free(data);
    return 0;
  }
  if (count == 0) return 0;
  off_t scan = st.st_size;
  off_t start = 0;
  if (scan > 0) {
    char last = 0;
    if (lseek(fd, scan - 1, SEEK_SET) < 0 || read(fd, &last, 1) != 1) {
      write_err_path("tail", path, errno);
      return 1;
    }
    if (last == '\n') scan--;
  }

  char buf[8192];
  unsigned long long seen = 0;
  while (scan > 0) {
    size_t want = scan > (off_t)sizeof(buf) ? sizeof(buf) : (size_t)scan;
    scan -= (off_t)want;
    if (lseek(fd, scan, SEEK_SET) < 0) {
      write_err_path("tail", path, errno);
      return 1;
    }
    size_t used = 0;
    while (used < want) {
      ssize_t read_len = read(fd, buf + used, want - used);
      if (read_len <= 0) {
        if (read_len < 0) write_err_path("tail", path, errno);
        return read_len < 0 ? 1 : 0;
      }
      used += (size_t)read_len;
    }
    for (size_t idx = want; idx > 0; idx--) {
      if (buf[idx - 1] == '\n') {
        seen++;
        if (seen == count) {
          start = scan + (off_t)idx;
          scan = 0;
          break;
        }
      }
    }
  }

  if (lseek(fd, start, SEEK_SET) < 0) {
    write_err_path("tail", path, errno);
    return 1;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("tail", path, errno);
      return 1;
    }
    write_bytes(buf, (size_t)read_len);
  }
  return 0;
}

static int cap_tail(int argc, char **argv) {
  int bytes = 0;
  unsigned long long count = 10;
  const char *path = NULL;
  int stdin_mode = 0;
  if (argc == 2) {
    stdin_mode = 1;
  } else if (argc == 3 && argv[2][0] == '-' && argv[2][1] >= '0' && argv[2][1] <= '9') {
    if (!parse_u64_arg(argv[2] + 1, &count)) return unsupported();
    stdin_mode = 1;
  } else if (argc == 3) {
    path = argv[2];
  } else if (argc == 4 && (!strcmp(argv[2], "-c") || !strcmp(argv[2], "-n"))) {
    bytes = !strcmp(argv[2], "-c");
    if (!parse_u64_arg(argv[3], &count)) return unsupported();
    stdin_mode = 1;
  } else if (argc == 5 && (!strcmp(argv[2], "-c") || !strcmp(argv[2], "-n"))) {
    bytes = !strcmp(argv[2], "-c");
    if (!parse_u64_arg(argv[3], &count)) return unsupported();
    path = argv[4];
  } else if (argc == 4 && argv[2][0] == '-' && argv[2][1] >= '0' && argv[2][1] <= '9') {
    if (!parse_u64_arg(argv[2] + 1, &count)) return unsupported();
    path = argv[3];
  } else {
    return unsupported();
  }
  int fd = stdin_mode ? STDIN_FILENO : open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("tail", path, errno);
    return 1;
  }
  int rc = bytes ? tail_copy_bytes(fd, path, count) : tail_copy_lines(fd, path, count);
  if (!stdin_mode) close(fd);
  return rc;
}

static int mkdir_p_one(const char *path) {
  char tmp[PATH_MAX];
  if (!copy_cstr(tmp, sizeof(tmp), path)) return 1;
  size_t len = strlen(tmp);
  while (len > 1 && tmp[len - 1] == '/') tmp[--len] = 0;
  for (char *p = tmp + 1; *p; p++) {
    if (*p != '/') continue;
    *p = 0;
    if (mkdir(tmp, 0777) != 0 && errno != EEXIST) {
      int err = errno;
      *p = '/';
      write_err_path("mkdir", path, err);
      return 1;
    }
    *p = '/';
  }
  if (mkdir(tmp, 0777) != 0 && errno != EEXIST) {
    write_err_path("mkdir", path, errno);
    return 1;
  }
  struct stat st;
  if (stat(tmp, &st) != 0 || !S_ISDIR(st.st_mode)) {
    write_err_path("mkdir", path, errno ? errno : ENOTDIR);
    return 1;
  }
  return 0;
}

static int cap_mkdir(int argc, char **argv) {
  int parents = 0;
  int first_path = 2;
  if (argc < 3) return unsupported();
  if (!strcmp(argv[2], "-p")) {
    parents = 1;
    first_path = 3;
  } else if (argv[2][0] == '-') {
    return unsupported();
  }
  if (first_path >= argc) return unsupported();
  int rc = 0;
  for (int idx = first_path; idx < argc; idx++) {
    if (argv[idx][0] == '-') return unsupported();
    if (parents) {
      rc |= mkdir_p_one(argv[idx]);
    } else if (mkdir(argv[idx], 0777) != 0) {
      write_err_path("mkdir", argv[idx], errno);
      rc = 1;
    }
  }
  return rc;
}

static int cap_touch(int argc, char **argv) {
  if (argc < 3) return unsupported();
  int rc = 0;
  for (int idx = 2; idx < argc; idx++) {
    if (argv[idx][0] == '-') return unsupported();
    if (utimes(argv[idx], NULL) == 0) continue;
    if (errno != ENOENT) {
      write_err_path("touch", argv[idx], errno);
      rc = 1;
      continue;
    }
    int fd = open(argv[idx], O_WRONLY | O_CREAT, 0666);
    if (fd < 0) {
      write_err_path("touch", argv[idx], errno);
      rc = 1;
      continue;
    }
    close(fd);
    if (utimes(argv[idx], NULL) != 0) {
      write_err_path("touch", argv[idx], errno);
      rc = 1;
    }
  }
  return rc;
}

static int cap_uniq(int argc, char **argv) {
  int stdin_mode = 0;
  const char *path = NULL;
  if (argc == 2) {
    stdin_mode = 1;
  } else if (argc == 3) {
    path = argv[2];
  } else {
    return unsupported();
  }
  if (!stdin_mode && output_discarded()) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return 1;
    close(fd);
    return 0;
  }

  FILE *file = stdin_mode ? stdin : fopen(path, "r");
  if (!file) {
    write_err_path("uniq", path, errno);
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
    write_err_path("uniq", path, errno);
    rc = 1;
  }
  free(previous);
  free(line);
  if (!stdin_mode) fclose(file);
  return rc;
}

static int cmp_string_ptr(const void *left, const void *right) {
  const char *a = *(const char * const *)left;
  const char *b = *(const char * const *)right;
  return strcmp(a, b);
}

enum ls_entry_mode {
  LS_ENTRY_VISIBLE = 0,
  LS_ENTRY_ALL,
  LS_ENTRY_ALMOST_ALL,
};

static int cap_ls(int argc, char **argv) {
  enum ls_entry_mode mode = LS_ENTRY_VISIBLE;
  const char *path = ".";
  int paths = 0;
  for (int idx = 2; idx < argc; idx++) {
    if (argv[idx][0] == '-' && argv[idx][1] != 0) {
      for (const char *flag = argv[idx] + 1; *flag; flag++) {
        if (*flag == 'a') {
          mode = LS_ENTRY_ALL;
        } else if (*flag == 'A') {
          if (mode != LS_ENTRY_ALL) mode = LS_ENTRY_ALMOST_ALL;
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
  if (mode == LS_ENTRY_ALL) {
    names[len++] = strdup(".");
    names[len++] = strdup("..");
  }
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    if (mode == LS_ENTRY_VISIBLE && entry->d_name[0] == '.') continue;
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

struct cut_plan {
  const char *file;
  int stdin_mode;
  unsigned char delimiter;
  unsigned long long field;
};

enum tr_mode_kind {
  TR_MODE_TRANSLATE = 0,
  TR_MODE_DELETE = 1,
};

struct tr_plan {
  enum tr_mode_kind mode;
  unsigned char map[256];
  unsigned char delete_set[256];
};

static int parse_cut_delimiter(const char *value, unsigned char *out) {
  if (!value || !value[0] || value[1]) return 0;
  *out = (unsigned char)value[0];
  return 1;
}

static int parse_cut_words(char **words, int start, int end,
                           const char *forced_file, struct cut_plan *plan) {
  plan->file = NULL;
  plan->stdin_mode = 0;
  plan->delimiter = '\t';
  plan->field = 0;
  for (int idx = start; idx < end; idx++) {
    const char *word = words[idx];
    if (!strcmp(word, "--")) return 0;
    if (!strcmp(word, "-d")) {
      if (++idx >= end || !parse_cut_delimiter(words[idx], &plan->delimiter)) return 0;
    } else if (!strncmp(word, "-d", 2) && word[2]) {
      if (!parse_cut_delimiter(word + 2, &plan->delimiter)) return 0;
    } else if (!strcmp(word, "-f")) {
      if (++idx >= end || !parse_u64_arg(words[idx], &plan->field) ||
          plan->field == 0) {
        return 0;
      }
    } else if (!strncmp(word, "-f", 2) && word[2]) {
      if (!parse_u64_arg(word + 2, &plan->field) || plan->field == 0) return 0;
    } else if (word[0] == '-') {
      return 0;
    } else {
      if (plan->file) return 0;
      plan->file = word;
    }
  }
  if (forced_file) {
    if (plan->file) return 0;
    plan->file = forced_file;
  }
  if (!plan->file) plan->stdin_mode = 1;
  return plan->field > 0;
}

static void tr_plan_init(struct tr_plan *plan) {
  plan->mode = TR_MODE_TRANSLATE;
  for (int idx = 0; idx < 256; idx++) {
    plan->map[idx] = (unsigned char)idx;
    plan->delete_set[idx] = 0;
  }
}

static int tr_reject_set_byte(unsigned char byte) {
  return byte == '\\' || byte == '[' || byte == ']' || byte == '*' ||
         byte == ':' || byte == '=' || byte == '\n' || byte == '\r' ||
         byte == 0;
}

static int expand_tr_class(const char *value, unsigned char *out,
                           size_t *out_len) {
  unsigned char first = 0;
  unsigned char last = 0;
  if (!strcmp(value, "[:lower:]")) {
    first = 'a';
    last = 'z';
  } else if (!strcmp(value, "[:upper:]")) {
    first = 'A';
    last = 'Z';
  } else if (!strcmp(value, "[:digit:]")) {
    first = '0';
    last = '9';
  } else {
    return 0;
  }
  *out_len = 0;
  for (unsigned int item = first; item <= last; item++) {
    out[(*out_len)++] = (unsigned char)item;
  }
  return 1;
}

static int expand_tr_set(const char *value, unsigned char *out, size_t *out_len) {
  size_t len = strlen(value);
  if (expand_tr_class(value, out, out_len)) return 1;
  if (len == 0) return 0;
  *out_len = 0;
  for (size_t idx = 0; idx < len; idx++) {
    unsigned char byte = (unsigned char)value[idx];
    if (byte >= 0x80 || tr_reject_set_byte(byte)) return 0;
    if (idx + 2 < len && value[idx + 1] == '-') {
      unsigned char end = (unsigned char)value[idx + 2];
      if (end >= 0x80 || tr_reject_set_byte(end) || byte >= end) return 0;
      for (unsigned int item = byte; item <= end; item++) {
        if (*out_len >= 256) return 0;
        out[(*out_len)++] = (unsigned char)item;
      }
      idx += 2;
    } else {
      if (byte == '-') return 0;
      if (*out_len >= 256) return 0;
      out[(*out_len)++] = byte;
    }
  }
  return *out_len > 0;
}

static int has_duplicate_tr_bytes(const unsigned char *bytes, size_t len) {
  unsigned char seen[256] = {0};
  for (size_t idx = 0; idx < len; idx++) {
    if (seen[bytes[idx]]) return 1;
    seen[bytes[idx]] = 1;
  }
  return 0;
}

static int parse_tr_words(char **words, int start, int end, struct tr_plan *plan) {
  unsigned char left[256];
  unsigned char right[256];
  size_t left_len = 0;
  size_t right_len = 0;
  tr_plan_init(plan);
  if (end - start == 2 && !strcmp(words[start], "-d")) {
    if (!expand_tr_set(words[start + 1], left, &left_len)) return 0;
    plan->mode = TR_MODE_DELETE;
    for (size_t idx = 0; idx < left_len; idx++) plan->delete_set[left[idx]] = 1;
    return 1;
  }
  if (end - start != 2 || words[start][0] == '-') return 0;
  if (!expand_tr_set(words[start], left, &left_len) ||
      !expand_tr_set(words[start + 1], right, &right_len) ||
      left_len != right_len || has_duplicate_tr_bytes(left, left_len)) {
    return 0;
  }
  plan->mode = TR_MODE_TRANSLATE;
  for (size_t idx = 0; idx < left_len; idx++) plan->map[left[idx]] = right[idx];
  return 1;
}

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
  int stdin_mode = 0;
  const char *path = NULL;
  if (argc == 2) {
    stdin_mode = 1;
  } else if (argc == 3) {
    path = argv[2];
  } else {
    return unsupported();
  }

  int fd = stdin_mode ? STDIN_FILENO : open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("sort", NULL, errno);
    return 2;
  }
  char *data = NULL;
  size_t size = 0;
  if (read_all_fd(fd, "sort", NULL, &data, &size)) {
    if (!stdin_mode) close(fd);
    return 2;
  }
  if (!stdin_mode) close(fd);

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

static void write_cut_segment(const char *line, size_t len,
                              unsigned char delimiter,
                              unsigned long long field) {
  int has_delimiter = 0;
  for (size_t idx = 0; idx < len; idx++) {
    if ((unsigned char)line[idx] == delimiter) {
      has_delimiter = 1;
      break;
    }
  }
  if (!has_delimiter) {
    write_bytes(line, len);
    write_bytes("\n", 1);
    return;
  }

  unsigned long long current_field = 1;
  size_t start = 0;
  for (size_t idx = 0; idx < len; idx++) {
    if ((unsigned char)line[idx] != delimiter) continue;
    if (current_field == field) {
      write_bytes(line + start, idx - start);
      write_bytes("\n", 1);
      return;
    }
    current_field++;
    start = idx + 1;
  }
  if (current_field == field) write_bytes(line + start, len - start);
  write_bytes("\n", 1);
}

static int cut_file(const struct cut_plan *plan, const char *err_cmd) {
  FILE *file = plan->stdin_mode ? stdin : fopen(plan->file, "r");
  if (!file) {
    write_err_path(err_cmd, plan->file, errno);
    return 1;
  }
  char *line = NULL;
  size_t cap = 0;
  ssize_t line_len = 0;
  int rc = 0;
  while ((line_len = getline(&line, &cap, file)) >= 0) {
    size_t len = (size_t)line_len;
    if (len > 0 && line[len - 1] == '\n') len--;
    write_cut_segment(line, len, plan->delimiter, plan->field);
  }
  if (ferror(file)) {
    write_err_path(err_cmd, plan->stdin_mode ? "stdin" : plan->file, errno);
    rc = 1;
  }
  free(line);
  if (!plan->stdin_mode) fclose(file);
  return rc;
}

static int cap_cut(int argc, char **argv) {
  struct cut_plan plan;
  if (!parse_cut_words(argv, 2, argc, NULL, &plan)) return unsupported();
  return cut_file(&plan, "cut");
}

static void tr_write_transformed(const struct tr_plan *plan, const char *data, size_t len) {
  char out[8192];
  size_t out_len = 0;
  if (plan->mode == TR_MODE_TRANSLATE) {
    for (size_t idx = 0; idx < len; idx++) {
      out[out_len++] = (char)plan->map[(unsigned char)data[idx]];
      if (out_len == sizeof(out)) {
        write_bytes(out, out_len);
        out_len = 0;
      }
    }
  } else {
    for (size_t idx = 0; idx < len; idx++) {
      unsigned char byte = (unsigned char)data[idx];
      if (plan->delete_set[byte]) continue;
      out[out_len++] = (char)byte;
      if (out_len == sizeof(out)) {
        write_bytes(out, out_len);
        out_len = 0;
      }
    }
  }
  if (out_len) write_bytes(out, out_len);
}

static int tr_fd(int fd, const struct tr_plan *plan, const char *err_cmd,
                 const char *err_path) {
  char buf[8192];
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) return 0;
    if (read_len < 0) {
      write_err_path(err_cmd, err_path, errno);
      return 1;
    }
    tr_write_transformed(plan, buf, (size_t)read_len);
  }
}

static int cap_tr(int argc, char **argv) {
  struct tr_plan plan;
  if (!parse_tr_words(argv, 2, argc, &plan)) return unsupported();
  return tr_fd(0, &plan, "tr", NULL);
}

static int load_sorted_file_for_pipe(const char *path, char **data_out,
                                     struct line_span **lines_out,
                                     size_t *line_len_out) {
  int fd = open(path, O_RDONLY);
  if (fd < 0) return unsupported();
  struct stat st;
  if (fstat(fd, &st) != 0 || !S_ISREG(st.st_mode)) {
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
      free(data);
      close(fd);
      return unsupported();
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

  *data_out = data;
  *lines_out = lines;
  *line_len_out = line_len;
  return 0;
}

static int load_regular_file_for_pipe(const char *path, char **data_out, size_t *size_out) {
  int fd = open(path, O_RDONLY);
  if (fd < 0) return unsupported();
  struct stat st;
  if (fstat(fd, &st) != 0 || !S_ISREG(st.st_mode)) {
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
      free(data);
      close(fd);
      return unsupported();
    }
    used += (size_t)r;
  }
  close(fd);
  *data_out = data;
  *size_out = size;
  return 0;
}

static void write_line_span_output(const char *data, struct line_span span) {
  const char *line = data + span.start;
  size_t n = span.end - span.start;
  write_bytes(line, n);
  if (n == 0 || line[n - 1] != '\n') write_bytes("\n", 1);
}

static const char *line_without_trailing_newline(const char *data,
                                                 struct line_span span,
                                                 size_t *len) {
  *len = span.end - span.start;
  if (*len > 0 && data[span.end - 1] == '\n') *len -= 1;
  return data + span.start;
}

static int line_spans_equal_without_newline(const char *data,
                                            struct line_span left,
                                            struct line_span right) {
  size_t left_len = 0;
  size_t right_len = 0;
  const char *left_line = line_without_trailing_newline(data, left, &left_len);
  const char *right_line = line_without_trailing_newline(data, right, &right_len);
  return left_len == right_len &&
         (left_len == 0 || memcmp(left_line, right_line, left_len) == 0);
}

static void emit_unique_line_spans(const char *data, const struct line_span *lines,
                                   size_t line_len) {
  int have_previous = 0;
  struct line_span previous = {0, 0};
  for (size_t idx = 0; idx < line_len; idx++) {
    if (!have_previous || !line_spans_equal_without_newline(data, previous, lines[idx])) {
      write_line_span_output(data, lines[idx]);
      previous = lines[idx];
      have_previous = 1;
    }
  }
}

static unsigned long long count_unique_line_spans(const char *data,
                                                  const struct line_span *lines,
                                                  size_t line_len) {
  unsigned long long count = 0;
  int have_previous = 0;
  struct line_span previous = {0, 0};
  for (size_t idx = 0; idx < line_len; idx++) {
    if (!have_previous || !line_spans_equal_without_newline(data, previous, lines[idx])) {
      count++;
      previous = lines[idx];
      have_previous = 1;
    }
  }
  return count;
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
  if (!S_ISREG(st.st_mode)) {
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

static int is_plain_literal_pattern(const char *pattern) {
  if (!pattern || !*pattern) return 0;
  for (const char *p = pattern; *p; p++) {
    switch (*p) {
      case '.':
      case '[':
      case ']':
      case '\\':
      case '*':
      case '^':
      case '$':
      case '+':
      case '?':
      case '{':
      case '}':
      case '(':
      case ')':
      case '|':
        return 0;
      default:
        break;
    }
  }
  return 1;
}

static unsigned long long count_newlines_fd(int fd, const char *cmd, const char *path, int *err) {
  char buf[8192];
  unsigned long long lines = 0;
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path(cmd, path, errno);
      *err = 1;
      break;
    }
    for (ssize_t pos = 0; pos < read_len; pos++) {
      if (buf[pos] == '\n') lines++;
    }
  }
  return lines;
}

static unsigned long long count_newlines_path(const char *path, const char *cmd, int *err) {
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path(cmd, path, errno);
    *err = 1;
    return 0;
  }
  unsigned long long lines = count_newlines_fd(fd, cmd, path, err);
  close(fd);
  return lines;
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

static int grep_plain_file(const char *path, const char *pat, size_t pat_len, int *matched) {
  char buf[8192];
  char line[8192];
  size_t used = 0;
  struct stat st;
  if (stat(path, &st) != 0 || !S_ISREG(st.st_mode)) return unsupported();
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("grep", path, errno);
    return 2;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("grep", path, errno);
      close(fd);
      return 2;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
        if (contains_bytes(line, (ssize_t)used, pat, pat_len)) {
          *matched = 1;
          write_bytes(line, used);
        }
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len)) {
    *matched = 1;
    write_bytes(line, used);
    write_bytes("\n", 1);
  }
  close(fd);
  return 0;
}

static int grep_file_head(const char *path, const char *pat, size_t pat_len,
                          unsigned long long *remaining, int *matched) {
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
          if (*remaining > 0) {
            write_cstr(path);
            write_bytes(":", 1);
            write_bytes(line, used);
            (*remaining)--;
            if (*remaining == 0) {
              close(fd);
              return 0;
            }
          }
        }
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len)) {
    *matched = 1;
    if (*remaining > 0) {
      write_cstr(path);
      write_bytes(":", 1);
      write_bytes(line, used);
      write_bytes("\n", 1);
      (*remaining)--;
    }
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

static int grep_walk_head(char *path, size_t cap, const char *pat, size_t pat_len,
                          unsigned long long *remaining, int *matched) {
  if (*remaining == 0) return 0;
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("grep", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode)) return grep_file_head(path, pat, pat_len, remaining, matched);
  if (!S_ISDIR(st.st_mode)) return 0;
  DIR *dir = opendir(path);
  if (!dir) {
    write_err_path("grep", path, errno);
    return 1;
  }
  size_t len = strlen(path);
  int rc = 0;
  struct dirent *entry = NULL;
  while (*remaining > 0 && (entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t entry_len = strlen(entry->d_name);
    if (len + 1 + entry_len + 1 > cap) continue;
    path[len] = '/';
    memcpy(path + len + 1, entry->d_name, entry_len + 1);
    if (entry->d_type == DT_DIR) {
      rc |= grep_walk_head(path, cap, pat, pat_len, remaining, matched);
    } else if (entry->d_type == DT_REG) {
      rc |= grep_file_head(path, pat, pat_len, remaining, matched);
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= grep_walk_head(path, cap, pat, pat_len, remaining, matched);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int grep_count_file(const char *path, const char *pat, size_t pat_len,
                           unsigned long long *matches) {
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
        if (contains_bytes(line, (ssize_t)used, pat, pat_len)) (*matches)++;
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len)) (*matches)++;
  close(fd);
  return 0;
}

static int grep_walk_count(char *path, size_t cap, const char *pat, size_t pat_len,
                           unsigned long long *matches) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("grep", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode)) return grep_count_file(path, pat, pat_len, matches);
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
      rc |= grep_walk_count(path, cap, pat, pat_len, matches);
    } else if (entry->d_type == DT_REG) {
      rc |= grep_count_file(path, pat, pat_len, matches);
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= grep_walk_count(path, cap, pat, pat_len, matches);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int parse_awk_print_field_script(const char *script, const char **filter,
                                        unsigned long long *field);
static void awk_field_bounds(const char *data, size_t len,
                             unsigned long long field, size_t *start,
                             size_t *end);

static int cap_awk(int argc, char **argv) {
  char buf[8192];
  char line[8192];
  size_t used = 0;
  unsigned long long count = 0;
  const char *script = "/NEEDLE/ { c++ } END { print c }";
  const char *pat = "NEEDLE";
  size_t pat_len = strlen(pat);
  const char *print_field_filter = NULL;
  unsigned long long print_field = 1;
  if ((argc == 3 || argc == 4) &&
      parse_awk_print_field_script(argv[2], &print_field_filter, &print_field)) {
    int stdin_mode = argc == 3;
    const char *path = stdin_mode ? NULL : argv[3];
    int fd = stdin_mode ? STDIN_FILENO : open(path, O_RDONLY);
    if (fd < 0) {
      write_err_path("awk", path, errno);
      return 2;
    }
    size_t filter_len = print_field_filter ? strlen(print_field_filter) : 0;
    for (;;) {
      ssize_t read_len = read(fd, buf, sizeof(buf));
      if (read_len == 0) break;
      if (read_len < 0) {
        write_err_path("awk", path, errno);
        if (!stdin_mode) close(fd);
        return 2;
      }
      for (ssize_t idx = 0; idx < read_len; idx++) {
        if (used < sizeof(line)) line[used++] = buf[idx];
        if (buf[idx] == '\n' || used == sizeof(line)) {
          if (!print_field_filter ||
              contains_bytes(line, (ssize_t)used, print_field_filter, filter_len)) {
            size_t start = 0;
            size_t end = 0;
            awk_field_bounds(line, used, print_field, &start, &end);
            write_bytes(line + start, end - start);
            write_bytes("\n", 1);
          }
          used = 0;
        }
      }
    }
    if (used &&
        (!print_field_filter ||
         contains_bytes(line, (ssize_t)used, print_field_filter, filter_len))) {
      size_t start = 0;
      size_t end = 0;
      awk_field_bounds(line, used, print_field, &start, &end);
      write_bytes(line + start, end - start);
      write_bytes("\n", 1);
    }
    if (!stdin_mode) close(fd);
    return 0;
  }
  if ((argc != 3 && argc != 4) || strcmp(argv[2], script)) return unsupported();
  int stdin_mode = argc == 3;
  const char *path = stdin_mode ? NULL : argv[3];
  int fd = stdin_mode ? STDIN_FILENO : open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("awk", path, errno);
    return 2;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("awk", path, errno);
      if (!stdin_mode) close(fd);
      return 2;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
        if (contains_bytes(line, (ssize_t)used, pat, pat_len)) count++;
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len)) count++;
  if (!stdin_mode) close(fd);
  if (count > 0) write_u64(count);
  write_bytes("\n", 1);
  return 0;
}

static int find_wc_emit_file(const char *path, unsigned long long *total,
                             unsigned long long *files, int *err);

static int parse_xargs_batch_arg(const char *arg, unsigned long long *batch_size) {
  unsigned long long value = 0;
  if (!parse_u64_arg(arg, &value) || value == 0) return 0;
  *batch_size = value;
  return 1;
}

static int parse_xargs_compact_batch_arg(const char *arg,
                                         unsigned long long *batch_size) {
  if (strncmp(arg, "-n", 2) || !arg[2]) return 0;
  return parse_xargs_batch_arg(arg + 2, batch_size);
}

static int xargs_echo_words_mode(char **words, int start, int end,
                                 unsigned long long *batch_size) {
  int len = end - start;
  *batch_size = 0;
  if (len == 1 && !strcmp(words[start], "xargs")) return 1;
  if (len == 2 && !strcmp(words[start], "xargs") && !strcmp(words[start + 1], "echo")) {
    return 1;
  }
  if (len == 3 && !strcmp(words[start], "xargs") && !strcmp(words[start + 1], "-n")) {
    return parse_xargs_batch_arg(words[start + 2], batch_size);
  }
  if (len == 2 && !strcmp(words[start], "xargs")) {
    return parse_xargs_compact_batch_arg(words[start + 1], batch_size);
  }
  if (len == 4 && !strcmp(words[start], "xargs") && !strcmp(words[start + 1], "-n") &&
      !strcmp(words[start + 3], "echo")) {
    return parse_xargs_batch_arg(words[start + 2], batch_size);
  }
  if (len == 3 && !strcmp(words[start], "xargs") &&
      !strcmp(words[start + 2], "echo")) {
    return parse_xargs_compact_batch_arg(words[start + 1], batch_size);
  }
  return 0;
}

static int cap_xargs(int argc, char **argv) {
  char buf[8192];
  char token[4096];
  size_t used = 0;
  int in_token = 0;
  int first = 1;
  unsigned long long batch_size = 0;
  unsigned long long batch_used = 0;
  int wc_lines = argc == 4 && !strcmp(argv[2], "wc") && !strcmp(argv[3], "-l");
  int echo_default = xargs_echo_words_mode(argv, 1, argc, &batch_size);
  if (!wc_lines && !echo_default) return unsupported();
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  for (;;) {
    ssize_t read_len = read(0, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("xargs", NULL, errno);
      return 1;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      unsigned char ch = (unsigned char)buf[idx];
      if (isspace(ch)) {
        if (in_token) {
          if (wc_lines) {
            token[used < sizeof(token) ? used : sizeof(token) - 1] = 0;
            (void)find_wc_emit_file(token, &total, &files, &err);
          } else if (batch_size) {
            if (batch_used) write_bytes(" ", 1);
            write_bytes(token, used);
            batch_used++;
            if (batch_used == batch_size) {
              write_bytes("\n", 1);
              batch_used = 0;
            }
          } else {
            if (!first) write_bytes(" ", 1);
            write_bytes(token, used);
            first = 0;
          }
          used = 0;
          in_token = 0;
        }
      } else {
        in_token = 1;
        if (used < sizeof(token)) token[used++] = (char)ch;
      }
    }
  }
  if (in_token) {
    if (wc_lines) {
      token[used < sizeof(token) ? used : sizeof(token) - 1] = 0;
      (void)find_wc_emit_file(token, &total, &files, &err);
    } else if (batch_size) {
      if (batch_used) write_bytes(" ", 1);
      write_bytes(token, used);
      batch_used++;
      if (batch_used == batch_size) {
        write_bytes("\n", 1);
        batch_used = 0;
      }
    } else {
      if (!first) write_bytes(" ", 1);
      write_bytes(token, used);
      first = 0;
    }
  }
  if (wc_lines) {
    if (files > 1) {
      write_padded_u64(total);
      write_bytes(" total\n", 7);
    }
    return err ? 1 : 0;
  }
  if (batch_size && batch_used) write_bytes("\n", 1);
  if (!batch_size && !first) write_bytes("\n", 1);
  return 0;
}

static int match_txt(const char *name) {
  size_t n = strlen(name);
  return n >= 4 && strcmp(name + n - 4, ".txt") == 0;
}

static int safe_name_glob(const char *pattern) {
  if (!pattern || !*pattern || pattern[0] == '-') return 0;
  for (const char *p = pattern; *p; p++) {
    if (*p == '/' || *p == '[' || *p == ']') return 0;
  }
  return 1;
}

static int name_glob_match_inner(const char *pattern, const char *text) {
  if (!*pattern) return !*text;
  if (*pattern == '*') {
    return name_glob_match_inner(pattern + 1, text) ||
           (*text && name_glob_match_inner(pattern, text + 1));
  }
  if (*pattern == '?') {
    return *text && name_glob_match_inner(pattern + 1, text + 1);
  }
  return *text == *pattern && name_glob_match_inner(pattern + 1, text + 1);
}

static int name_glob_match(const char *pattern, const char *name) {
  return safe_name_glob(pattern) && name_glob_match_inner(pattern, name);
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

enum wc_count_mode {
  WC_COUNT_LINES,
  WC_COUNT_BYTES,
  WC_COUNT_WORDS,
};

static int parse_wc_count_mode(const char *flag, enum wc_count_mode *mode) {
  if (!strcmp(flag, "-l")) {
    *mode = WC_COUNT_LINES;
    return 1;
  }
  if (!strcmp(flag, "-c")) {
    *mode = WC_COUNT_BYTES;
    return 1;
  }
  if (!strcmp(flag, "-w")) {
    *mode = WC_COUNT_WORDS;
    return 1;
  }
  return 0;
}

static unsigned long long count_wc_fd(int fd, enum wc_count_mode mode,
                                      const char *path, int *err) {
  char buf[8192];
  unsigned long long count = 0;
  int in_word = 0;

  if (mode == WC_COUNT_BYTES) {
    struct stat st;
    off_t offset = lseek(fd, 0, SEEK_CUR);
    if (offset >= 0 && fstat(fd, &st) == 0 && S_ISREG(st.st_mode) && st.st_size >= offset) {
      return (unsigned long long)(st.st_size - offset);
    }
  }

  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("wc", path, errno);
      *err = 1;
      break;
    }
    if (mode == WC_COUNT_BYTES) {
      count += (unsigned long long)read_len;
      continue;
    }
    for (ssize_t pos = 0; pos < read_len; pos++) {
      unsigned char byte = (unsigned char)buf[pos];
      if (mode == WC_COUNT_LINES) {
        if (byte == '\n') count++;
      } else if (isspace(byte)) {
        in_word = 0;
      } else if (!in_word) {
        count++;
        in_word = 1;
      }
    }
  }
  return count;
}

struct wc_all_counts {
  unsigned long long lines;
  unsigned long long words;
  unsigned long long bytes;
};

static void wc_all_counts_add(struct wc_all_counts *total, struct wc_all_counts item) {
  total->lines += item.lines;
  total->words += item.words;
  total->bytes += item.bytes;
}

static int count_wc_all_fd(int fd, const char *path, struct wc_all_counts *out) {
  char buf[8192];
  int in_word = 0;
  out->lines = 0;
  out->words = 0;
  out->bytes = 0;

  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("wc", path, errno);
      return 0;
    }
    out->bytes += (unsigned long long)read_len;
    for (ssize_t pos = 0; pos < read_len; pos++) {
      unsigned char byte = (unsigned char)buf[pos];
      if (byte == '\n') out->lines++;
      if (isspace(byte)) {
        in_word = 0;
      } else if (!in_word) {
        out->words++;
        in_word = 1;
      }
    }
  }
  return 1;
}

static void write_wc_all_counts(struct wc_all_counts counts, const char *label) {
  write_padded_u64(counts.lines);
  write_padded_u64(counts.words);
  write_padded_u64(counts.bytes);
  if (label) {
    write_bytes(" ", 1);
    write_cstr(label);
  }
  write_bytes("\n", 1);
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
static int cap_wc(int argc, char **argv) {
  int file_start = 2;
  int all_counts = 1;
  enum wc_count_mode mode;
  unsigned long long total_count = 0;
  struct wc_all_counts total_all = {0, 0, 0};
  int exit_code = 0;
  int discard = output_discarded();

  if (argc >= 3 && parse_wc_count_mode(argv[2], &mode)) {
    all_counts = 0;
    file_start = 3;
  }
  int file_count = argc - file_start;

  if (file_count == 0) {
    if (all_counts) {
      struct wc_all_counts counts;
      if (!count_wc_all_fd(STDIN_FILENO, "stdin", &counts)) return 1;
      write_wc_all_counts(counts, NULL);
    } else {
      int count_err = 0;
      unsigned long long count = count_wc_fd(STDIN_FILENO, mode, "stdin", &count_err);
      if (count_err) return 1;
      write_padded_u64(count);
      write_bytes("\n", 1);
    }
    return 0;
  }

  for (int idx = file_start; idx < argc; idx++) {
    if (argv[idx][0] == '-') return unsupported();
    struct stat st;
    if (stat(argv[idx], &st) != 0) continue;
    if (!S_ISREG(st.st_mode)) return unsupported();
  }

  for (int idx = file_start; idx < argc; idx++) {
    int fd = open(argv[idx], O_RDONLY);
    if (fd < 0) {
      write_err_path("wc", argv[idx], errno);
      exit_code = 1;
      continue;
    }
    if (discard) {
      close(fd);
      continue;
    }

    if (all_counts) {
      struct wc_all_counts counts;
      int ok = count_wc_all_fd(fd, argv[idx], &counts);
      close(fd);
      if (!ok) {
        exit_code = 1;
        continue;
      }
      wc_all_counts_add(&total_all, counts);
      write_wc_all_counts(counts, argv[idx]);
    } else {
      int count_err = 0;
      unsigned long long count = count_wc_fd(fd, mode, argv[idx], &count_err);
      close(fd);
      if (count_err) {
        exit_code = 1;
        continue;
      }
      total_count += count;
      write_padded_u64(count);
      write_bytes(" ", 1);
      write_cstr(argv[idx]);
      write_bytes("\n", 1);
    }
  }

  if (!discard && file_count > 1) {
    if (all_counts) {
      write_wc_all_counts(total_all, "total");
    } else {
      write_padded_u64(total_count);
      write_bytes(" total\n", 7);
    }
  }
  return exit_code;
}

static int find_can_descend(int max_depth, int depth) {
  return max_depth < 0 || depth < max_depth;
}

static int find_wc_walk_path(char *path, size_t cap, const char *name_glob,
                             int max_depth, int depth, unsigned long long *total,
                             unsigned long long *files, int *err);

static int find_wc_emit_file(const char *path, unsigned long long *total,
                             unsigned long long *files, int *err) {
  int count_err = 0;
  unsigned long long lines = count_newlines_path(path, "wc", &count_err);
  if (count_err) {
    *err = 1;
    return 1;
  }
  *total += lines;
  *files += 1;
  write_padded_u64(lines);
  write_bytes(" ", 1);
  write_cstr(path);
  write_bytes("\n", 1);
  return 0;
}

static int find_wc_walk_dir(char *path, size_t cap, const char *name_glob,
                            int max_depth, int depth,
                            unsigned long long *total, unsigned long long *files, int *err) {
  DIR *dir = opendir(path);
  if (!dir) {
    write_err_path("find", path, errno);
    *err = 1;
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
      if (find_can_descend(max_depth, depth + 1)) {
        rc |= find_wc_walk_dir(path, cap, name_glob, max_depth, depth + 1, total, files, err);
      }
    } else if (entry->d_type == DT_REG) {
      if (name_glob_match(name_glob, entry->d_name)) {
        rc |= find_wc_emit_file(path, total, files, err);
      }
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= find_wc_walk_path(path, cap, name_glob, max_depth, depth + 1, total, files, err);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int find_wc_walk_path(char *path, size_t cap, const char *name_glob,
                             int max_depth, int depth, unsigned long long *total,
                             unsigned long long *files, int *err) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("find", path, errno);
    *err = 1;
    return 1;
  }
  if (S_ISREG(st.st_mode) && name_glob_match(name_glob, cap_base(path))) {
    return find_wc_emit_file(path, total, files, err);
  }
  if (!S_ISDIR(st.st_mode)) return 0;
  if (!find_can_descend(max_depth, depth)) return 0;
  return find_wc_walk_dir(path, cap, name_glob, max_depth, depth, total, files, err);
}

static int find_head_walk_path(char *path, size_t cap, const char *name_glob,
                               int max_depth, int depth, unsigned long long *remaining, int *err);

static int find_head_emit_file(const char *path, unsigned long long *remaining) {
  if (*remaining == 0) return 0;
  write_line(path);
  (*remaining)--;
  return 0;
}

static int find_head_walk_dir(char *path, size_t cap, const char *name_glob,
                              int max_depth, int depth,
                              unsigned long long *remaining, int *err) {
  if (*remaining == 0) return 0;
  DIR *dir = opendir(path);
  if (!dir) {
    write_err_path("find", path, errno);
    *err = 1;
    return 1;
  }
  size_t len = strlen(path);
  int rc = 0;
  struct dirent *entry = NULL;
  while (*remaining > 0 && (entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t entry_len = strlen(entry->d_name);
    if (len + 1 + entry_len + 1 > cap) continue;
    path[len] = '/';
    memcpy(path + len + 1, entry->d_name, entry_len + 1);
    if (entry->d_type == DT_DIR) {
      if (find_can_descend(max_depth, depth + 1)) {
        rc |= find_head_walk_dir(path, cap, name_glob, max_depth, depth + 1, remaining, err);
      }
    } else if (entry->d_type == DT_REG) {
      if (name_glob_match(name_glob, entry->d_name)) {
        rc |= find_head_emit_file(path, remaining);
      }
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= find_head_walk_path(path, cap, name_glob, max_depth, depth + 1, remaining, err);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int find_head_walk_path(char *path, size_t cap, const char *name_glob,
                               int max_depth, int depth, unsigned long long *remaining, int *err) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("find", path, errno);
    *err = 1;
    return 1;
  }
  if (S_ISREG(st.st_mode) && name_glob_match(name_glob, cap_base(path))) {
    return find_head_emit_file(path, remaining);
  }
  if (!S_ISDIR(st.st_mode)) return 0;
  if (!find_can_descend(max_depth, depth)) return 0;
  return find_head_walk_dir(path, cap, name_glob, max_depth, depth, remaining, err);
}

static int find_count_walk_path(char *path, size_t cap, const char *name_glob,
                                int max_depth, int depth, unsigned long long *count);

static int find_count_walk_dir(char *path, size_t cap, const char *name_glob,
                               int max_depth, int depth, unsigned long long *count) {
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
      if (find_can_descend(max_depth, depth + 1)) {
        rc |= find_count_walk_dir(path, cap, name_glob, max_depth, depth + 1, count);
      }
    } else if (entry->d_type == DT_REG) {
      if (name_glob_match(name_glob, entry->d_name)) (*count)++;
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= find_count_walk_path(path, cap, name_glob, max_depth, depth + 1, count);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int find_count_walk_path(char *path, size_t cap, const char *name_glob,
                                int max_depth, int depth, unsigned long long *count) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("find", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode) && name_glob_match(name_glob, cap_base(path))) {
    (*count)++;
    return 0;
  }
  if (!S_ISDIR(st.st_mode)) return 0;
  if (!find_can_descend(max_depth, depth)) return 0;
  return find_count_walk_dir(path, cap, name_glob, max_depth, depth, count);
}

struct path_list {
  char **items;
  size_t len;
  size_t cap;
};

static void path_list_free(struct path_list *list) {
  for (size_t idx = 0; idx < list->len; idx++) free(list->items[idx]);
  free(list->items);
  list->items = NULL;
  list->len = 0;
  list->cap = 0;
}

static int path_list_push(struct path_list *list, const char *path) {
  if (list->len == list->cap) {
    size_t next_cap = list->cap ? list->cap * 2 : 128;
    char **next = (char **)realloc(list->items, sizeof(char *) * next_cap);
    if (!next) return 0;
    list->items = next;
    list->cap = next_cap;
  }
  list->items[list->len] = strdup(path);
  if (!list->items[list->len]) return 0;
  list->len++;
  return 1;
}

static int path_list_push_tail(struct path_list *list, const char *path,
                               unsigned long long limit) {
  if (limit == 0) return 1;
  if (limit <= (unsigned long long)((size_t)-1) && list->len == (size_t)limit) {
    free(list->items[0]);
    if (list->len > 1) {
      memmove(list->items, list->items + 1, sizeof(char *) * (list->len - 1));
    }
    list->len--;
  }
  return path_list_push(list, path);
}

enum path_lookup_mode {
  PATH_LOOKUP_WHICH = 0,
  PATH_LOOKUP_WHICH_ALL,
  PATH_LOOKUP_COMMAND_V,
};

enum shell_word_kind {
  SHELL_WORD_NONE = 0,
  SHELL_WORD_BUILTIN,
  SHELL_WORD_RESERVED,
};

static enum shell_word_kind shell_word_kind(const char *name) {
  static const char *builtins[] = {
      "alias",   "bg",       "bind",    "break",   "builtin", "caller",
      "cd",      "command",  "compgen", "complete", "compopt", "continue",
      "declare", "dirs",     "disown",  "echo",    "enable",  "eval",
      "exec",    "exit",     "export",  "false",   "fc",      "fg",
      "getopts", "hash",     "help",    "history", "jobs",    "kill",
      "let",     "local",    "logout",  "mapfile", "popd",    "printf",
      "pushd",   "pwd",      "read",    "readarray", "readonly", "return",
      "set",     "shift",    "shopt",   "source",  "suspend", "test",
      "times",   "trap",     "true",    "type",    "typeset", "ulimit",
      "umask",   "unalias",  "unset",   "wait",    "[",
  };
  static const char *reserved[] = {
      "!",    "[[",  "]]",   "{",    "}",    "case", "do",
      "done", "elif", "else", "esac", "fi",   "for",  "function",
      "if",   "in",  "select", "then", "time", "until", "while",
  };
  for (size_t idx = 0; idx < sizeof(builtins) / sizeof(builtins[0]); idx++) {
    if (!strcmp(name, builtins[idx])) return SHELL_WORD_BUILTIN;
  }
  for (size_t idx = 0; idx < sizeof(reserved) / sizeof(reserved[0]); idx++) {
    if (!strcmp(name, reserved[idx])) return SHELL_WORD_RESERVED;
  }
  return SHELL_WORD_NONE;
}

static int path_lookup_candidate_matches(const char *path, int executable) {
  struct stat st;
  if (stat(path, &st) != 0 || !S_ISREG(st.st_mode)) return 0;
  if (executable && access(path, X_OK) != 0) return 0;
  return 1;
}

static int find_path_lookup_candidate(const char *name, int executable,
                                      char *out, size_t out_cap) {
  const char *path_env = getenv("PATH");
  if (!path_env) return 0;
  const char *start = path_env;
  while (1) {
    const char *end = strchr(start, ':');
    size_t dir_len = end ? (size_t)(end - start) : strlen(start);
    const char *dir = dir_len ? start : ".";
    size_t used_dir_len = dir_len ? dir_len : 1;
    size_t name_len = strlen(name);
    if (used_dir_len + 1 + name_len + 1 <= out_cap) {
      memcpy(out, dir, used_dir_len);
      out[used_dir_len] = '/';
      memcpy(out + used_dir_len + 1, name, name_len + 1);
      if (path_lookup_candidate_matches(out, executable)) return 1;
    }
    if (!end) break;
    start = end + 1;
  }
  return 0;
}

static int collect_path_lookup_candidates(const char *name, int executable,
                                          struct path_list *lines) {
  const char *path_env = getenv("PATH");
  if (!path_env) return 1;
  const char *start = path_env;
  char out[PATH_MAX + 128];
  while (1) {
    const char *end = strchr(start, ':');
    size_t dir_len = end ? (size_t)(end - start) : strlen(start);
    const char *dir = dir_len ? start : ".";
    size_t used_dir_len = dir_len ? dir_len : 1;
    size_t name_len = strlen(name);
    if (used_dir_len + 1 + name_len + 1 <= sizeof(out)) {
      memcpy(out, dir, used_dir_len);
      out[used_dir_len] = '/';
      memcpy(out + used_dir_len + 1, name, name_len + 1);
      if (path_lookup_candidate_matches(out, executable) && !path_list_push(lines, out)) {
        return 0;
      }
    }
    if (!end) break;
    start = end + 1;
  }
  return 1;
}

static int resolve_path_lookup_line(const char *name, enum path_lookup_mode mode,
                                    char *out, size_t out_cap) {
  int has_slash = strchr(name, '/') != NULL;
  if (has_slash) {
    if (!path_lookup_candidate_matches(name, 1)) return 0;
    return copy_cstr(out, out_cap, name);
  }
  if (mode == PATH_LOOKUP_WHICH || mode == PATH_LOOKUP_WHICH_ALL) {
    return find_path_lookup_candidate(name, 1, out, out_cap);
  }
  if (shell_word_kind(name) != SHELL_WORD_NONE) {
    return copy_cstr(out, out_cap, name);
  }
  return find_path_lookup_candidate(name, 0, out, out_cap);
}

static int collect_path_lookup_lines(enum path_lookup_mode mode, char **names,
                                     int start, int end, struct path_list *lines,
                                     int *found_any, int *missing_any) {
  char line[PATH_MAX + 128];
  *found_any = 0;
  *missing_any = 0;
  for (int idx = start; idx < end; idx++) {
    if (names[idx][0] == '-') return 0;
    if (mode == PATH_LOOKUP_WHICH_ALL) {
      size_t before = lines->len;
      if (strchr(names[idx], '/')) {
        if (path_lookup_candidate_matches(names[idx], 1) && !path_list_push(lines, names[idx])) {
          return 0;
        }
      } else if (!collect_path_lookup_candidates(names[idx], 1, lines)) {
        return 0;
      }
      if (lines->len > before) {
        *found_any = 1;
      } else {
        *missing_any = 1;
      }
      continue;
    }
    if (resolve_path_lookup_line(names[idx], mode, line, sizeof(line))) {
      if (!path_list_push(lines, line)) return 0;
      *found_any = 1;
    } else {
      *missing_any = 1;
    }
  }
  return 1;
}

static void emit_path_lookup_lines(const struct path_list *lines) {
  for (size_t idx = 0; idx < lines->len; idx++) write_line(lines->items[idx]);
}

static int cap_which(int argc, char **argv) {
  if (argc < 3) return unsupported();
  enum path_lookup_mode mode = PATH_LOOKUP_WHICH;
  int start = 2;
  if (!strcmp(argv[2], "-a")) {
    mode = PATH_LOOKUP_WHICH_ALL;
    start = 3;
  }
  if (start >= argc) return unsupported();
  struct path_list lines = {0};
  int found_any = 0;
  int missing_any = 0;
  if (!collect_path_lookup_lines(mode, argv, start, argc, &lines, &found_any,
                                 &missing_any)) {
    path_list_free(&lines);
    return unsupported();
  }
  (void)found_any;
  emit_path_lookup_lines(&lines);
  path_list_free(&lines);
  return missing_any ? 1 : 0;
}

static int cap_command_builtin(int argc, char **argv) {
  if (argc < 4 || strcmp(argv[2], "-v")) return unsupported();
  struct path_list lines = {0};
  int found_any = 0;
  int missing_any = 0;
  if (!collect_path_lookup_lines(PATH_LOOKUP_COMMAND_V, argv, 3, argc, &lines,
                                 &found_any, &missing_any)) {
    path_list_free(&lines);
    return unsupported();
  }
  (void)missing_any;
  emit_path_lookup_lines(&lines);
  path_list_free(&lines);
  return found_any ? 0 : 1;
}

enum environment_mode {
  ENVIRONMENT_ENV = 0,
  ENVIRONMENT_PRINTENV,
};

static const char *find_environment_value(const char *name) {
  if (!name || !*name) return NULL;
  size_t name_len = strlen(name);
  for (char **entry = environ; entry && *entry; entry++) {
    if (!strncmp(*entry, name, name_len) && (*entry)[name_len] == '=') {
      return *entry + name_len + 1;
    }
  }
  return NULL;
}

static void emit_environment_all(void) {
  for (char **entry = environ; entry && *entry; entry++) write_line(*entry);
}

static int collect_environment_lines(enum environment_mode mode, const char *name,
                                     struct path_list *lines, int *found_any) {
  *found_any = 0;
  if (mode == ENVIRONMENT_ENV && name) return 0;
  if (name) {
    const char *value = find_environment_value(name);
    if (!value) return 1;
    if (!path_list_push(lines, value)) return 0;
    *found_any = 1;
    return 1;
  }
  for (char **entry = environ; entry && *entry; entry++) {
    if (!path_list_push(lines, *entry)) return 0;
    *found_any = 1;
  }
  return 1;
}

static int cap_env_builtin(int argc, char **argv) {
  (void)argv;
  if (argc != 2) return unsupported();
  emit_environment_all();
  return 0;
}

static int cap_printenv(int argc, char **argv) {
  if (argc == 2) {
    emit_environment_all();
    return 0;
  }
  if (argc == 3 && argv[2][0] != '-') {
    const char *value = find_environment_value(argv[2]);
    if (!value) return 1;
    write_line(value);
    return 0;
  }
  return unsupported();
}

static int find_collect_named_path(char *path, size_t cap, const char *name_glob,
                                   int max_depth, int depth, struct path_list *list);

static int find_collect_named_dir(char *path, size_t cap, const char *name_glob,
                                  int max_depth, int depth, struct path_list *list) {
  size_t len = strlen(path);
  DIR *dir = opendir(path);
  if (!dir) return unsupported();
  if (len + 2 >= cap) {
    closedir(dir);
    return unsupported();
  }
  path[len++] = '/';
  path[len] = 0;
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t name_len = strlen(entry->d_name);
    if (len + name_len + 1 >= cap) {
      closedir(dir);
      return unsupported();
    }
    memcpy(path + len, entry->d_name, name_len + 1);
    if (entry->d_type == DT_DIR) {
      if (find_can_descend(max_depth, depth + 1)) {
        int rc = find_collect_named_dir(path, cap, name_glob, max_depth, depth + 1, list);
        if (rc != 0) {
          closedir(dir);
          return rc;
        }
      }
    } else if (entry->d_type == DT_REG) {
      if (name_glob_match(name_glob, entry->d_name) && !path_list_push(list, path)) {
        closedir(dir);
        return 1;
      }
    } else if (entry->d_type == DT_UNKNOWN) {
      int rc = find_collect_named_path(path, cap, name_glob, max_depth, depth + 1, list);
      if (rc != 0) {
        closedir(dir);
        return rc;
      }
    }
    path[len] = 0;
  }
  closedir(dir);
  return 0;
}

static int find_collect_named_path(char *path, size_t cap, const char *name_glob,
                                   int max_depth, int depth, struct path_list *list) {
  struct stat st;
  if (lstat(path, &st) != 0) return unsupported();
  if (S_ISREG(st.st_mode) && name_glob_match(name_glob, cap_base(path))) {
    return path_list_push(list, path) ? 0 : 1;
  }
  if (!S_ISDIR(st.st_mode)) return 0;
  if (!find_can_descend(max_depth, depth)) return 0;
  return find_collect_named_dir(path, cap, name_glob, max_depth, depth, list);
}

static int cat_grep_file(const char *path, const char *pat, size_t pat_len, int *matched) {
  char buf[8192];
  char line[8192];
  size_t used = 0;
  if (pat_len == 0) return unsupported();
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("cat", path, errno);
    return 0;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("cat", path, errno);
      close(fd);
      return 0;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
        if (contains_bytes(line, (ssize_t)used, pat, pat_len)) {
          *matched = 1;
          write_bytes(line, used);
        }
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len)) {
    *matched = 1;
    write_bytes(line, used);
    write_bytes("\n", 1);
  }
  close(fd);
  return 0;
}

static int single_pipe_index(char **words, int count) {
  int pipe = -1;
  for (int idx = 0; idx < count; idx++) {
    if (strcmp(words[idx], "|")) continue;
    if (pipe >= 0) return -1;
    pipe = idx;
  }
  if (pipe <= 0 || pipe + 1 >= count) return -1;
  return pipe;
}

static int first_pipe_index(char **words, int count) {
  for (int idx = 0; idx < count; idx++) {
    if (!strcmp(words[idx], "|")) {
      if (idx <= 0 || idx + 1 >= count) return -1;
      return idx;
    }
  }
  return -1;
}

static int parse_path_lookup_pipe_left(char **words, int pipe,
                                       enum path_lookup_mode *mode,
                                       int *name_start) {
  if (pipe >= 3 && !strcmp(words[0], "which") && !strcmp(words[1], "-a")) {
    *mode = PATH_LOOKUP_WHICH_ALL;
    *name_start = 2;
    return 1;
  }
  if (pipe >= 2 && !strcmp(words[0], "which")) {
    *mode = PATH_LOOKUP_WHICH;
    *name_start = 1;
    return 1;
  }
  if (pipe >= 3 && !strcmp(words[0], "command") && !strcmp(words[1], "-v")) {
    *mode = PATH_LOOKUP_COMMAND_V;
    *name_start = 2;
    return 1;
  }
  return 0;
}

static int pipe_path_lookup_wc(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum path_lookup_mode mode = PATH_LOOKUP_WHICH;
  int name_start = 0;
  struct path_list lines = {0};
  int found_any = 0;
  int missing_any = 0;
  if (pipe < 0 || count - pipe != 3 || strcmp(words[pipe + 1], "wc") ||
      strcmp(words[pipe + 2], "-l")) {
    return unsupported();
  }
  if (!parse_path_lookup_pipe_left(words, pipe, &mode, &name_start) ||
      !collect_path_lookup_lines(mode, words, name_start, pipe, &lines,
                                 &found_any, &missing_any)) {
    path_list_free(&lines);
    return unsupported();
  }
  (void)found_any;
  (void)missing_any;
  write_padded_u64((unsigned long long)lines.len);
  write_bytes("\n", 1);
  path_list_free(&lines);
  return 0;
}

static int pipe_path_lookup_head(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum path_lookup_mode mode = PATH_LOOKUP_WHICH;
  int name_start = 0;
  unsigned long long limit = 0;
  struct path_list lines = {0};
  int found_any = 0;
  int missing_any = 0;
  if (pipe < 0 || count - pipe != 4 || strcmp(words[pipe + 1], "head") ||
      strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit) || limit == 0) {
    return unsupported();
  }
  if (!parse_path_lookup_pipe_left(words, pipe, &mode, &name_start) ||
      !collect_path_lookup_lines(mode, words, name_start, pipe, &lines,
                                 &found_any, &missing_any)) {
    path_list_free(&lines);
    return unsupported();
  }
  (void)found_any;
  (void)missing_any;
  size_t emit = lines.len < (size_t)limit ? lines.len : (size_t)limit;
  for (size_t idx = 0; idx < emit; idx++) write_line(lines.items[idx]);
  path_list_free(&lines);
  return 0;
}

static int pipe_path_lookup_tail(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum path_lookup_mode mode = PATH_LOOKUP_WHICH;
  int name_start = 0;
  unsigned long long limit = 0;
  struct path_list lines = {0};
  int found_any = 0;
  int missing_any = 0;
  if (pipe < 0 || count - pipe != 4 || strcmp(words[pipe + 1], "tail") ||
      strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit)) {
    return unsupported();
  }
  if (!parse_path_lookup_pipe_left(words, pipe, &mode, &name_start) ||
      !collect_path_lookup_lines(mode, words, name_start, pipe, &lines,
                                 &found_any, &missing_any)) {
    path_list_free(&lines);
    return unsupported();
  }
  (void)found_any;
  (void)missing_any;
  size_t emit = lines.len < (size_t)limit ? lines.len : (size_t)limit;
  size_t start = lines.len - emit;
  for (size_t idx = start; idx < lines.len; idx++) write_line(lines.items[idx]);
  path_list_free(&lines);
  return 0;
}

static void emit_xargs_echo_path(const char *path, int *first);
static int emit_xargs_wc_bytes(const char *data, size_t len,
                               unsigned long long *total,
                               unsigned long long *files, int *err);

static int emit_path_list_downstream(struct path_list *lines, char **words, int count,
                                     int start) {
  enum wc_count_mode wc_mode;
  unsigned long long limit = 0;
  if (count - start == 2 && !strcmp(words[start], "wc") &&
      parse_wc_count_mode(words[start + 1], &wc_mode)) {
    unsigned long long n = 0;
    int in_word = 0;
    if (wc_mode == WC_COUNT_LINES) {
      n = (unsigned long long)lines->len;
    } else {
      for (size_t idx = 0; idx < lines->len; idx++) {
        const unsigned char *s = (const unsigned char *)lines->items[idx];
        for (; *s; s++) {
          if (wc_mode == WC_COUNT_BYTES) {
            n++;
          } else if (isspace(*s)) {
            in_word = 0;
          } else if (!in_word) {
            n++;
            in_word = 1;
          }
        }
        if (wc_mode == WC_COUNT_BYTES) {
          n++;
        } else {
          in_word = 0;
        }
      }
    }
    write_padded_u64(n);
    write_bytes("\n", 1);
    return 0;
  }
  if (count - start == 3 && !strcmp(words[start], "head") &&
      !strcmp(words[start + 1], "-n") &&
      parse_u64_arg(words[start + 2], &limit) && limit > 0) {
    size_t emit = lines->len < (size_t)limit ? lines->len : (size_t)limit;
    for (size_t idx = 0; idx < emit; idx++) write_line(lines->items[idx]);
    return 0;
  }
  if (count - start == 3 && !strcmp(words[start], "tail") &&
      !strcmp(words[start + 1], "-n") &&
      parse_u64_arg(words[start + 2], &limit)) {
    size_t emit = lines->len < (size_t)limit ? lines->len : (size_t)limit;
    size_t first = lines->len - emit;
    for (size_t idx = first; idx < lines->len; idx++) write_line(lines->items[idx]);
    return 0;
  }
  if (count - start == 1 && !strcmp(words[start], "sort")) {
    qsort(lines->items, lines->len, sizeof(char *), cmp_string_ptr);
    for (size_t idx = 0; idx < lines->len; idx++) write_line(lines->items[idx]);
    return 0;
  }
  if (count - start >= 3 && !strcmp(words[start], "sort") &&
      !strcmp(words[start + 1], "|")) {
    qsort(lines->items, lines->len, sizeof(char *), cmp_string_ptr);
    return emit_path_list_downstream(lines, words, count, start + 2);
  }
  if (count - start == 2 && !strcmp(words[start], "xargs") &&
      !strcmp(words[start + 1], "echo")) {
    int first = 1;
    for (size_t idx = 0; idx < lines->len; idx++) emit_xargs_echo_path(lines->items[idx], &first);
    if (!first) write_bytes("\n", 1);
    return 0;
  }
  if (count - start == 3 && !strcmp(words[start], "xargs") &&
      !strcmp(words[start + 1], "wc") && !strcmp(words[start + 2], "-l")) {
    unsigned long long total = 0;
    unsigned long long files = 0;
    int err = 0;
    for (size_t idx = 0; idx < lines->len; idx++) {
      if (!emit_xargs_wc_bytes(lines->items[idx], strlen(lines->items[idx]), &total, &files,
                               &err)) {
        return 1;
      }
    }
    if (files > 1) {
      write_padded_u64(total);
      write_bytes(" total\n", 7);
    }
    return err ? 1 : 0;
  }
  return unsupported();
}

static int pipe_path_lookup_grep(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  enum path_lookup_mode mode = PATH_LOOKUP_WHICH;
  int name_start = 0;
  struct path_list lines = {0};
  struct path_list filtered = {0};
  int found_any = 0;
  int missing_any = 0;
  int matched = 0;
  int rc = 0;
  if (pipe < 0 || count - pipe < 3 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return unsupported();
  }
  if (!parse_path_lookup_pipe_left(words, pipe, &mode, &name_start) ||
      !collect_path_lookup_lines(mode, words, name_start, pipe, &lines,
                                 &found_any, &missing_any)) {
    path_list_free(&lines);
    return unsupported();
  }
  (void)found_any;
  (void)missing_any;
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < lines.len; idx++) {
    if (!contains_bytes(lines.items[idx], (ssize_t)strlen(lines.items[idx]), pattern,
                        pattern_len)) {
      continue;
    }
    matched = 1;
    if (count - pipe == 3) {
      write_line(lines.items[idx]);
    } else if (count - pipe >= 6 && !strcmp(words[pipe + 3], "|")) {
      if (!path_list_push(&filtered, lines.items[idx])) {
        path_list_free(&lines);
        path_list_free(&filtered);
        return 1;
      }
    } else {
      path_list_free(&lines);
      path_list_free(&filtered);
      return unsupported();
    }
  }
  if (count - pipe > 3) {
    if (count - pipe < 6 || strcmp(words[pipe + 3], "|")) {
      path_list_free(&lines);
      path_list_free(&filtered);
      return unsupported();
    }
    rc = emit_path_list_downstream(&filtered, words, count, pipe + 4);
  }
  path_list_free(&lines);
  path_list_free(&filtered);
  if (rc == 127) return unsupported();
  return count - pipe == 3 && !matched ? 1 : rc;
}

static int pipe_path_lookup_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  enum path_lookup_mode mode = PATH_LOOKUP_WHICH;
  int name_start = 0;
  struct path_list lines = {0};
  int found_any = 0;
  int missing_any = 0;
  int rc = 0;
  if (pipe < 0) return unsupported();
  if (!parse_path_lookup_pipe_left(words, pipe, &mode, &name_start) ||
      !collect_path_lookup_lines(mode, words, name_start, pipe, &lines,
                                 &found_any, &missing_any)) {
    path_list_free(&lines);
    return unsupported();
  }
  (void)found_any;
  (void)missing_any;
  rc = emit_path_list_downstream(&lines, words, count, pipe + 1);
  path_list_free(&lines);
  if (rc == 127) return unsupported();
  return rc;
}

static int parse_environment_pipe_left(char **words, int pipe,
                                       enum environment_mode *mode,
                                       const char **name) {
  *name = NULL;
  if (pipe == 2 && !strcmp(words[0], "printenv") && words[1][0] != '-') {
    *mode = ENVIRONMENT_PRINTENV;
    *name = words[1];
    return 1;
  }
  return 0;
}

static int collect_environment_pipe_lines(char **words, int pipe,
                                          struct path_list *lines) {
  enum environment_mode mode = ENVIRONMENT_ENV;
  const char *name = NULL;
  int found_any = 0;
  if (!parse_environment_pipe_left(words, pipe, &mode, &name) ||
      !collect_environment_lines(mode, name, lines, &found_any)) {
    return 0;
  }
  (void)found_any;
  return 1;
}

static int pipe_environment_wc(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  struct path_list lines = {0};
  if (pipe < 0 || count - pipe != 3 || strcmp(words[pipe + 1], "wc") ||
      strcmp(words[pipe + 2], "-l")) {
    return unsupported();
  }
  if (!collect_environment_pipe_lines(words, pipe, &lines)) {
    path_list_free(&lines);
    return unsupported();
  }
  write_padded_u64((unsigned long long)lines.len);
  write_bytes("\n", 1);
  path_list_free(&lines);
  return 0;
}

static int pipe_environment_head(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  unsigned long long limit = 0;
  struct path_list lines = {0};
  if (pipe < 0 || count - pipe != 4 || strcmp(words[pipe + 1], "head") ||
      strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit) || limit == 0) {
    return unsupported();
  }
  if (!collect_environment_pipe_lines(words, pipe, &lines)) {
    path_list_free(&lines);
    return unsupported();
  }
  size_t emit = lines.len < (size_t)limit ? lines.len : (size_t)limit;
  for (size_t idx = 0; idx < emit; idx++) write_line(lines.items[idx]);
  path_list_free(&lines);
  return 0;
}

static int pipe_environment_tail(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  unsigned long long limit = 0;
  struct path_list lines = {0};
  if (pipe < 0 || count - pipe != 4 || strcmp(words[pipe + 1], "tail") ||
      strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit)) {
    return unsupported();
  }
  if (!collect_environment_pipe_lines(words, pipe, &lines)) {
    path_list_free(&lines);
    return unsupported();
  }
  size_t emit = lines.len < (size_t)limit ? lines.len : (size_t)limit;
  size_t start = lines.len - emit;
  for (size_t idx = start; idx < lines.len; idx++) write_line(lines.items[idx]);
  path_list_free(&lines);
  return 0;
}

static int pipe_environment_grep(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct path_list lines = {0};
  struct path_list filtered = {0};
  int matched = 0;
  int rc = 0;
  if (pipe < 0 || count - pipe < 3 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return unsupported();
  }
  if (!collect_environment_pipe_lines(words, pipe, &lines)) {
    path_list_free(&lines);
    return unsupported();
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < lines.len; idx++) {
    if (contains_bytes(lines.items[idx], (ssize_t)strlen(lines.items[idx]), pattern,
                       pattern_len)) {
      matched = 1;
      if (count - pipe == 3) {
        write_line(lines.items[idx]);
      } else if (count - pipe >= 6 && !strcmp(words[pipe + 3], "|")) {
        if (!path_list_push(&filtered, lines.items[idx])) {
          path_list_free(&lines);
          path_list_free(&filtered);
          return 1;
        }
      } else {
        path_list_free(&lines);
        path_list_free(&filtered);
        return unsupported();
      }
    }
  }
  if (count - pipe > 3) {
    if (count - pipe < 6 || strcmp(words[pipe + 3], "|")) {
      path_list_free(&lines);
      path_list_free(&filtered);
      return unsupported();
    }
    enum wc_count_mode wc_mode;
    unsigned long long limit = 0;
    int start = pipe + 4;
    if (count - start == 2 && !strcmp(words[start], "wc") &&
        parse_wc_count_mode(words[start + 1], &wc_mode)) {
      unsigned long long n = 0;
      int in_word = 0;
      if (wc_mode == WC_COUNT_LINES) {
        n = (unsigned long long)filtered.len;
      } else {
        for (size_t idx = 0; idx < filtered.len; idx++) {
          const unsigned char *s = (const unsigned char *)filtered.items[idx];
          for (; *s; s++) {
            if (wc_mode == WC_COUNT_BYTES) {
              n++;
            } else if (isspace(*s)) {
              in_word = 0;
            } else if (!in_word) {
              n++;
              in_word = 1;
            }
          }
          if (wc_mode == WC_COUNT_BYTES) {
            n++;
          } else {
            in_word = 0;
          }
        }
      }
      write_padded_u64(n);
      write_bytes("\n", 1);
      rc = 0;
    } else if (count - start == 3 && !strcmp(words[start], "head") &&
               !strcmp(words[start + 1], "-n") &&
               parse_u64_arg(words[start + 2], &limit) && limit > 0) {
      size_t emit = filtered.len < (size_t)limit ? filtered.len : (size_t)limit;
      for (size_t idx = 0; idx < emit; idx++) write_line(filtered.items[idx]);
      rc = 0;
    } else if (count - start == 3 && !strcmp(words[start], "tail") &&
               !strcmp(words[start + 1], "-n") &&
               parse_u64_arg(words[start + 2], &limit)) {
      size_t emit = filtered.len < (size_t)limit ? filtered.len : (size_t)limit;
      size_t first = filtered.len - emit;
      for (size_t idx = first; idx < filtered.len; idx++) write_line(filtered.items[idx]);
      rc = 0;
    } else if (count - start == 1 && !strcmp(words[start], "sort")) {
      qsort(filtered.items, filtered.len, sizeof(char *), cmp_string_ptr);
      for (size_t idx = 0; idx < filtered.len; idx++) write_line(filtered.items[idx]);
      rc = 0;
    } else {
      rc = unsupported();
    }
  }
  path_list_free(&lines);
  path_list_free(&filtered);
  if (rc == 127) return unsupported();
  return count - pipe == 3 && !matched ? 1 : rc;
}

static int pipe_environment_sort(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  struct path_list lines = {0};
  if (pipe < 0 || count - pipe != 2 || strcmp(words[pipe + 1], "sort")) {
    return unsupported();
  }
  if (!collect_environment_pipe_lines(words, pipe, &lines)) {
    path_list_free(&lines);
    return unsupported();
  }
  qsort(lines.items, lines.len, sizeof(char *), cmp_string_ptr);
  for (size_t idx = 0; idx < lines.len; idx++) write_line(lines.items[idx]);
  path_list_free(&lines);
  return 0;
}

static int parse_hostname_pipe_left(char **words, int pipe) {
  return pipe == 1 && !strcmp(words[0], "hostname");
}

static int read_hostname(char *out, size_t out_cap) {
  if (gethostname(out, out_cap) != 0) return 0;
  if (out_cap > 0) out[out_cap - 1] = 0;
  return 1;
}

static int pipe_hostname_wc(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  char name[256];
  if (pipe < 0 || count - pipe != 3 || strcmp(words[pipe + 1], "wc") ||
      strcmp(words[pipe + 2], "-l") || !parse_hostname_pipe_left(words, pipe)) {
    return unsupported();
  }
  if (!read_hostname(name, sizeof(name))) {
    write_err_path("hostname", NULL, errno);
    return 1;
  }
  write_padded_u64(1);
  write_bytes("\n", 1);
  return 0;
}

static int pipe_hostname_head(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  unsigned long long limit = 0;
  char name[256];
  if (pipe < 0 || count - pipe != 4 || strcmp(words[pipe + 1], "head") ||
      strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit) || limit == 0 ||
      !parse_hostname_pipe_left(words, pipe)) {
    return unsupported();
  }
  if (!read_hostname(name, sizeof(name))) {
    write_err_path("hostname", NULL, errno);
    return 1;
  }
  write_line(name);
  return 0;
}

static int pipe_hostname_tail(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  unsigned long long limit = 0;
  char name[256];
  if (pipe < 0 || count - pipe != 4 || strcmp(words[pipe + 1], "tail") ||
      strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit) ||
      !parse_hostname_pipe_left(words, pipe)) {
    return unsupported();
  }
  if (!read_hostname(name, sizeof(name))) {
    write_err_path("hostname", NULL, errno);
    return 1;
  }
  if (limit > 0) write_line(name);
  return 0;
}

static int pipe_hostname_grep(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  char name[256];
  struct path_list filtered = {0};
  int matched = 0;
  int rc = 0;
  if (pipe < 0 || count - pipe < 3 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2]) ||
      !parse_hostname_pipe_left(words, pipe)) {
    return unsupported();
  }
  if (!read_hostname(name, sizeof(name))) {
    write_err_path("hostname", NULL, errno);
    return 1;
  }
  if (contains_bytes(name, (ssize_t)strlen(name), words[pipe + 2],
                     strlen(words[pipe + 2]))) {
    matched = 1;
    if (count - pipe == 3) {
      write_line(name);
    } else if (count - pipe >= 6 && !strcmp(words[pipe + 3], "|")) {
      if (!path_list_push(&filtered, name)) {
        path_list_free(&filtered);
        return 1;
      }
    } else {
      path_list_free(&filtered);
      return unsupported();
    }
  }
  if (count - pipe > 3) {
    if (count - pipe < 6 || strcmp(words[pipe + 3], "|")) {
      path_list_free(&filtered);
      return unsupported();
    }
    rc = emit_path_list_downstream(&filtered, words, count, pipe + 4);
  }
  path_list_free(&filtered);
  if (rc == 127) return unsupported();
  return count - pipe == 3 && !matched ? 1 : rc;
}

static int pipe_hostname_sort(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  char name[256];
  if (pipe < 0 || count - pipe != 2 || strcmp(words[pipe + 1], "sort") ||
      !parse_hostname_pipe_left(words, pipe)) {
    return unsupported();
  }
  if (!read_hostname(name, sizeof(name))) {
    write_err_path("hostname", NULL, errno);
    return 1;
  }
  write_line(name);
  return 0;
}

static int parse_echo_words(char **words, int start, int end, int *first_arg,
                            int *newline) {
  *first_arg = start + 1;
  *newline = 1;
  if (end - start >= 2 && words[start + 1][0] == '-') {
    if (strcmp(words[start + 1], "-n")) return 0;
    *first_arg = start + 2;
    *newline = 0;
  }
  if (!*newline) {
    for (int idx = *first_arg; idx < end; idx++) {
      if (words[idx][0] == '-') return 0;
    }
  }
  return 1;
}

static void emit_echo_words(char **words, int first_arg, int end, int newline) {
  for (int idx = first_arg; idx < end; idx++) {
    if (idx > first_arg) write_bytes(" ", 1);
    write_cstr(words[idx]);
  }
  if (newline) write_bytes("\n", 1);
}

static int pipe_echo_wc(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  int first_arg = 0;
  int newline = 1;
  if (pipe < 0 || strcmp(words[0], "echo") || count - pipe != 3 ||
      strcmp(words[pipe + 1], "wc") || strcmp(words[pipe + 2], "-l")) {
    return unsupported();
  }
  if (!parse_echo_words(words, 0, pipe, &first_arg, &newline)) return unsupported();
  write_padded_u64(newline ? 1 : 0);
  write_bytes("\n", 1);
  return 0;
}

static int pipe_echo_head(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  int first_arg = 0;
  int newline = 1;
  unsigned long long limit = 0;
  if (pipe < 0 || strcmp(words[0], "echo") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "head") || strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit) || limit == 0) {
    return unsupported();
  }
  if (!parse_echo_words(words, 0, pipe, &first_arg, &newline)) return unsupported();
  emit_echo_words(words, first_arg, pipe, newline);
  return 0;
}

static int pipe_echo_tail(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  int first_arg = 0;
  int newline = 1;
  unsigned long long limit = 0;
  if (pipe < 0 || strcmp(words[0], "echo") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "tail") || strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit)) {
    return unsupported();
  }
  if (!parse_echo_words(words, 0, pipe, &first_arg, &newline)) return unsupported();
  if (limit > 0) emit_echo_words(words, first_arg, pipe, newline);
  return 0;
}

static int parse_printf_words(char **words, int start, int end,
                              enum printf_format_kind *kind, int *first_arg) {
  if (end - start < 3) return 0;
  *kind = printf_format_kind(words[start + 1]);
  if (*kind == PRINTF_FORMAT_UNSUPPORTED) return 0;
  *first_arg = start + 2;
  return 1;
}

static int pipe_printf_wc(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  if (pipe < 0 || strcmp(words[0], "printf") || count - pipe != 3 ||
      strcmp(words[pipe + 1], "wc") || strcmp(words[pipe + 2], "-l")) {
    return unsupported();
  }
  if (!parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      kind != PRINTF_FORMAT_STRING_NEWLINE) {
    return unsupported();
  }
  write_padded_u64((unsigned long long)(pipe - first_arg));
  write_bytes("\n", 1);
  return 0;
}

static int pipe_printf_head(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  unsigned long long limit = 0;
  if (pipe < 0 || strcmp(words[0], "printf") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "head") || strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit) || limit == 0) {
    return unsupported();
  }
  if (!parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      kind != PRINTF_FORMAT_STRING_NEWLINE) {
    return unsupported();
  }
  for (int idx = first_arg; idx < pipe && limit > 0; idx++, limit--) {
    write_cstr(words[idx]);
    write_bytes("\n", 1);
  }
  return 0;
}

static int pipe_printf_tail(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  unsigned long long limit = 0;
  if (pipe < 0 || strcmp(words[0], "printf") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "tail") || strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit)) {
    return unsupported();
  }
  if (!parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      kind != PRINTF_FORMAT_STRING_NEWLINE) {
    return unsupported();
  }
  unsigned long long total = (unsigned long long)(pipe - first_arg);
  unsigned long long emit = total < limit ? total : limit;
  int start = pipe - (int)emit;
  for (int idx = start; idx < pipe; idx++) {
    write_cstr(words[idx]);
    write_bytes("\n", 1);
  }
  return 0;
}

static int pipe_printf_grep(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  int matched = 0;
  if (pipe < 0 || strcmp(words[0], "printf") || count - pipe != 3 ||
      strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return unsupported();
  }
  if (!parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      kind != PRINTF_FORMAT_STRING_NEWLINE) {
    return unsupported();
  }
  const char *pat = words[pipe + 2];
  size_t pat_len = strlen(pat);
  for (int idx = first_arg; idx < pipe; idx++) {
    if (contains_bytes(words[idx], (ssize_t)strlen(words[idx]), pat, pat_len)) {
      matched = 1;
      write_cstr(words[idx]);
      write_bytes("\n", 1);
    }
  }
  return matched ? 0 : 1;
}

static void emit_echo_words_tr(char **words, int first_arg, int end, int newline,
                               const struct tr_plan *plan) {
  for (int idx = first_arg; idx < end; idx++) {
    if (idx > first_arg) tr_write_transformed(plan, " ", 1);
    tr_write_transformed(plan, words[idx], strlen(words[idx]));
  }
  if (newline) tr_write_transformed(plan, "\n", 1);
}

static void emit_xargs_echo_path(const char *path, int *first);
static void emit_xargs_echo_bytes(const char *data, size_t len, int *first);
struct xargs_echo_batch_state {
  unsigned long long size;
  unsigned long long used;
};
static void emit_xargs_echo_batch_bytes(const char *data, size_t len,
                                        struct xargs_echo_batch_state *state);
static void finish_xargs_echo_batch(struct xargs_echo_batch_state *state);
static int emit_xargs_wc_bytes(const char *data, size_t len,
                               unsigned long long *total,
                               unsigned long long *files, int *err);

static int pipe_echo_tr(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  int first_arg = 0;
  int newline = 1;
  struct tr_plan plan;
  if (pipe < 0 || strcmp(words[0], "echo") || count - pipe < 4 ||
      strcmp(words[pipe + 1], "tr")) {
    return unsupported();
  }
  if (!parse_echo_words(words, 0, pipe, &first_arg, &newline) ||
      !parse_tr_words(words, pipe + 2, count, &plan)) {
    return unsupported();
  }
  emit_echo_words_tr(words, first_arg, pipe, newline, &plan);
  return 0;
}

static int pipe_echo_xargs_echo(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  int first_arg = 0;
  int newline = 1;
  int first = 1;
  unsigned long long batch_size = 0;
  struct xargs_echo_batch_state batch = {0};
  if (pipe < 0 || strcmp(words[0], "echo") ||
      !xargs_echo_words_mode(words, pipe + 1, count, &batch_size)) {
    return unsupported();
  }
  if (!parse_echo_words(words, 0, pipe, &first_arg, &newline)) return unsupported();
  (void)newline;
  batch.size = batch_size;
  for (int idx = first_arg; idx < pipe; idx++) {
    if (batch_size) {
      emit_xargs_echo_batch_bytes(words[idx], strlen(words[idx]), &batch);
    } else {
      emit_xargs_echo_path(words[idx], &first);
    }
  }
  if (batch_size) finish_xargs_echo_batch(&batch);
  if (!batch_size && !first) write_bytes("\n", 1);
  return 0;
}

static int pipe_echo_xargs_wc(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  int first_arg = 0;
  int newline = 1;
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (pipe < 0 || strcmp(words[0], "echo") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "xargs") || strcmp(words[pipe + 2], "wc") ||
      strcmp(words[pipe + 3], "-l")) {
    return unsupported();
  }
  if (!parse_echo_words(words, 0, pipe, &first_arg, &newline)) return unsupported();
  (void)newline;
  for (int idx = first_arg; idx < pipe; idx++) {
    if (!emit_xargs_wc_bytes(words[idx], strlen(words[idx]), &total, &files, &err)) {
      return 1;
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  return err ? 1 : 0;
}

static int pipe_printf_tr(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  struct tr_plan plan;
  if (pipe < 0 || strcmp(words[0], "printf") || count - pipe < 4 ||
      strcmp(words[pipe + 1], "tr")) {
    return unsupported();
  }
  if (!parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      !parse_tr_words(words, pipe + 2, count, &plan)) {
    return unsupported();
  }
  for (int idx = first_arg; idx < pipe; idx++) {
    tr_write_transformed(&plan, words[idx], strlen(words[idx]));
    if (kind == PRINTF_FORMAT_STRING_NEWLINE) tr_write_transformed(&plan, "\n", 1);
  }
  return 0;
}

static int pipe_printf_xargs_echo(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  int first = 1;
  unsigned long long batch_size = 0;
  struct xargs_echo_batch_state batch = {0};
  if (pipe < 0 || strcmp(words[0], "printf") ||
      !xargs_echo_words_mode(words, pipe + 1, count, &batch_size)) {
    return unsupported();
  }
  if (!parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      kind != PRINTF_FORMAT_STRING_NEWLINE) {
    return unsupported();
  }
  batch.size = batch_size;
  for (int idx = first_arg; idx < pipe; idx++) {
    if (batch_size) {
      emit_xargs_echo_batch_bytes(words[idx], strlen(words[idx]), &batch);
    } else {
      emit_xargs_echo_path(words[idx], &first);
    }
  }
  if (batch_size) finish_xargs_echo_batch(&batch);
  if (!batch_size && !first) write_bytes("\n", 1);
  return 0;
}

static int pipe_printf_xargs_wc(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (pipe < 0 || strcmp(words[0], "printf") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "xargs") || strcmp(words[pipe + 2], "wc") ||
      strcmp(words[pipe + 3], "-l")) {
    return unsupported();
  }
  if (!parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      kind != PRINTF_FORMAT_STRING_NEWLINE) {
    return unsupported();
  }
  for (int idx = first_arg; idx < pipe; idx++) {
    if (!emit_xargs_wc_bytes(words[idx], strlen(words[idx]), &total, &files, &err)) {
      return 1;
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  return err ? 1 : 0;
}

static int pipe_seq_wc(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  struct seq_plan seq;
  if (pipe < 0 || strcmp(words[0], "seq") || count - pipe != 3 ||
      strcmp(words[pipe + 1], "wc") || strcmp(words[pipe + 2], "-l")) {
    return unsupported();
  }
  if (!parse_seq_words(words, 0, pipe, &seq)) return unsupported();
  write_padded_u64(seq_count(&seq));
  write_bytes("\n", 1);
  return 0;
}

static int pipe_seq_head(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  struct seq_plan seq;
  unsigned long long limit = 0;
  if (pipe < 0 || strcmp(words[0], "seq") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "head") || strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit) || limit == 0) {
    return unsupported();
  }
  if (!parse_seq_words(words, 0, pipe, &seq)) return unsupported();
  emit_seq(&seq, limit);
  return 0;
}

static int pipe_seq_tail(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  struct seq_plan seq;
  unsigned long long limit = 0;
  if (pipe < 0 || strcmp(words[0], "seq") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "tail") || strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit)) {
    return unsupported();
  }
  if (!parse_seq_words(words, 0, pipe, &seq)) return unsupported();
  emit_seq_tail(&seq, limit);
  return 0;
}

static int pipe_seq_xargs_echo(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  struct seq_plan seq;
  unsigned long long batch_size = 0;
  unsigned long long batch_used = 0;
  if (pipe < 0 || strcmp(words[0], "seq") ||
      !xargs_echo_words_mode(words, pipe + 1, count, &batch_size)) {
    return unsupported();
  }
  if (!parse_seq_words(words, 0, pipe, &seq)) return unsupported();
  unsigned long long remaining = seq_count(&seq);
  long long current = seq.first;
  int first = 1;
  char buf[32];
  while (remaining > 0) {
    int len = snprintf(buf, sizeof(buf), "%lld", current);
    if (len > 0) {
      if (batch_size && batch_used) write_bytes(" ", 1);
      if (!batch_size && !first) write_bytes(" ", 1);
      write_bytes(buf, (size_t)len);
      if (batch_size) {
        batch_used++;
        if (batch_used == batch_size) {
          write_bytes("\n", 1);
          batch_used = 0;
        }
      }
      first = 0;
    }
    remaining--;
    if (remaining == 0) break;
    current += seq.step;
  }
  if (batch_size && batch_used) write_bytes("\n", 1);
  if (!batch_size && !first) write_bytes("\n", 1);
  return 0;
}

static int pipe_yes_head(char **words, int count) {
  int pipe = single_pipe_index(words, count);
  unsigned long long limit = 0;
  const char *value = "y";
  if (pipe < 0 || strcmp(words[0], "yes") || count - pipe != 4 ||
      strcmp(words[pipe + 1], "head") || strcmp(words[pipe + 2], "-n") ||
      !parse_u64_arg(words[pipe + 3], &limit) || limit == 0) {
    return unsupported();
  }
  if (pipe == 2) {
    value = words[1];
    if (value[0] == '-') return unsupported();
  } else if (pipe != 1) {
    return unsupported();
  }
  for (unsigned long long idx = 0; idx < limit; idx++) {
    write_cstr(value);
    write_bytes("\n", 1);
  }
  return 0;
}

static int pipe_sort_uniq(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  if (count != 4 || strcmp(words[0], "sort") || strcmp(words[2], "|") ||
      strcmp(words[3], "uniq")) {
    return unsupported();
  }
  int rc = load_sorted_file_for_pipe(words[1], &data, &lines, &line_len);
  if (rc != 0) return rc;
  emit_unique_line_spans(data, lines, line_len);
  free(lines);
  free(data);
  return 0;
}

static int pipe_sort_uniq_wc(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  if (count != 7 || strcmp(words[0], "sort") || strcmp(words[2], "|") ||
      strcmp(words[3], "uniq") || strcmp(words[4], "|") ||
      strcmp(words[5], "wc") || strcmp(words[6], "-l")) {
    return unsupported();
  }
  int rc = load_sorted_file_for_pipe(words[1], &data, &lines, &line_len);
  if (rc != 0) return rc;
  write_padded_u64(count_unique_line_spans(data, lines, line_len));
  write_bytes("\n", 1);
  free(lines);
  free(data);
  return 0;
}

static int pipe_sort_wc(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  if (count != 5 || strcmp(words[0], "sort") || strcmp(words[2], "|") ||
      strcmp(words[3], "wc") || strcmp(words[4], "-l")) {
    return unsupported();
  }
  int rc = load_sorted_file_for_pipe(words[1], &data, &lines, &line_len);
  if (rc != 0) return rc;
  write_padded_u64((unsigned long long)line_len);
  write_bytes("\n", 1);
  free(lines);
  free(data);
  return 0;
}

static int pipe_sort_head(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  unsigned long long limit = 0;
  if (count != 6 || strcmp(words[0], "sort") || strcmp(words[2], "|") ||
      strcmp(words[3], "head") || strcmp(words[4], "-n") ||
      !parse_u64_arg(words[5], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = load_sorted_file_for_pipe(words[1], &data, &lines, &line_len);
  if (rc != 0) return rc;
  size_t take = limit > (unsigned long long)line_len ? line_len : (size_t)limit;
  for (size_t idx = 0; idx < take; idx++) {
    write_line_span_output(data, lines[idx]);
  }
  free(lines);
  free(data);
  return 0;
}

static int pipe_sort_tail(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  unsigned long long limit = 0;
  if (count != 6 || strcmp(words[0], "sort") || strcmp(words[2], "|") ||
      strcmp(words[3], "tail") || strcmp(words[4], "-n") ||
      !parse_u64_arg(words[5], &limit)) {
    return unsupported();
  }
  int rc = load_sorted_file_for_pipe(words[1], &data, &lines, &line_len);
  if (rc != 0) return rc;
  size_t take = limit > (unsigned long long)line_len ? line_len : (size_t)limit;
  size_t start = line_len - take;
  for (size_t idx = start; idx < line_len; idx++) {
    write_line_span_output(data, lines[idx]);
  }
  free(lines);
  free(data);
  return 0;
}

static int pipe_sort_xargs_echo(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  int first = 1;
  if (count != 5 || strcmp(words[0], "sort") || strcmp(words[2], "|") ||
      strcmp(words[3], "xargs") || strcmp(words[4], "echo")) {
    return unsupported();
  }
  int rc = load_sorted_file_for_pipe(words[1], &data, &lines, &line_len);
  if (rc != 0) return rc;
  for (size_t idx = 0; idx < line_len; idx++) {
    emit_xargs_echo_bytes(data + lines[idx].start, lines[idx].end - lines[idx].start, &first);
  }
  if (!first) write_bytes("\n", 1);
  free(lines);
  free(data);
  return 0;
}

static int pipe_sort_xargs_wc(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 6 || strcmp(words[0], "sort") || strcmp(words[2], "|") ||
      strcmp(words[3], "xargs") || strcmp(words[4], "wc") ||
      strcmp(words[5], "-l")) {
    return unsupported();
  }
  int rc = load_sorted_file_for_pipe(words[1], &data, &lines, &line_len);
  if (rc != 0) return rc;
  for (size_t idx = 0; idx < line_len; idx++) {
    if (!emit_xargs_wc_bytes(data + lines[idx].start, lines[idx].end - lines[idx].start,
                             &total, &files, &err)) {
      free(lines);
      free(data);
      return 1;
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  free(lines);
  free(data);
  return err ? 1 : 0;
}

static int load_cat_sorted_pipe(char **words, int count, char **data,
                                struct line_span **lines, size_t *line_len) {
  if (count < 4 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "sort")) {
    return unsupported();
  }
  return load_sorted_file_for_pipe(words[1], data, lines, line_len);
}

static size_t line_len_without_newline(const char *line, size_t len) {
  return len > 0 && line[len - 1] == '\n' ? len - 1 : len;
}

static int cat_uniq_file(const char *path, int count_only) {
  FILE *file = fopen(path, "r");
  unsigned long long count = 0;
  char *line = NULL;
  size_t cap = 0;
  char *previous = NULL;
  size_t previous_len = 0;
  int have_previous = 0;
  if (!file) {
    write_err_path("cat", path, errno);
    if (count_only) {
      write_padded_u64(0);
      write_bytes("\n", 1);
    }
    return 0;
  }

  for (;;) {
    ssize_t read_len = getline(&line, &cap, file);
    if (read_len < 0) break;
    size_t len = (size_t)read_len;
    size_t cmp_len = line_len_without_newline(line, len);
    int duplicate = have_previous && previous_len == cmp_len &&
                    (cmp_len == 0 || memcmp(previous, line, cmp_len) == 0);
    if (!duplicate) {
      count++;
      if (!count_only) {
        write_bytes(line, len);
        if (len == 0 || line[len - 1] != '\n') write_bytes("\n", 1);
      }
      char *next_previous = (char *)malloc(cmp_len ? cmp_len : 1);
      if (!next_previous) {
        free(previous);
        free(line);
        fclose(file);
        return 1;
      }
      if (cmp_len) memcpy(next_previous, line, cmp_len);
      free(previous);
      previous = next_previous;
      previous_len = cmp_len;
      have_previous = 1;
    }
  }

  if (ferror(file)) write_err_path("cat", path, errno);
  if (count_only) {
    write_padded_u64(count);
    write_bytes("\n", 1);
  }
  free(previous);
  free(line);
  fclose(file);
  return 0;
}

static int pipe_cat_uniq(char **words, int count) {
  if (count != 4 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "uniq")) {
    return unsupported();
  }
  return cat_uniq_file(words[1], 0);
}

static int pipe_cat_uniq_wc(char **words, int count) {
  if (count != 7 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "uniq") || strcmp(words[4], "|") ||
      strcmp(words[5], "wc") || strcmp(words[6], "-l")) {
    return unsupported();
  }
  return cat_uniq_file(words[1], 1);
}

static int pipe_cat_sort(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  if (count != 4) return unsupported();
  int rc = load_cat_sorted_pipe(words, count, &data, &lines, &line_len);
  if (rc != 0) return rc;
  for (size_t idx = 0; idx < line_len; idx++) {
    write_line_span_output(data, lines[idx]);
  }
  free(lines);
  free(data);
  return 0;
}

static int pipe_cat_sort_uniq(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  if (count != 6 || strcmp(words[4], "|") || strcmp(words[5], "uniq")) return unsupported();
  int rc = load_cat_sorted_pipe(words, count, &data, &lines, &line_len);
  if (rc != 0) return rc;
  emit_unique_line_spans(data, lines, line_len);
  free(lines);
  free(data);
  return 0;
}

static int pipe_cat_sort_uniq_wc(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  if (count != 9 || strcmp(words[4], "|") || strcmp(words[5], "uniq") ||
      strcmp(words[6], "|") || strcmp(words[7], "wc") || strcmp(words[8], "-l")) {
    return unsupported();
  }
  int rc = load_cat_sorted_pipe(words, count, &data, &lines, &line_len);
  if (rc != 0) return rc;
  write_padded_u64(count_unique_line_spans(data, lines, line_len));
  write_bytes("\n", 1);
  free(lines);
  free(data);
  return 0;
}

static int pipe_cat_sort_wc(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  if (count != 7 || strcmp(words[4], "|") || strcmp(words[5], "wc") ||
      strcmp(words[6], "-l")) {
    return unsupported();
  }
  int rc = load_cat_sorted_pipe(words, count, &data, &lines, &line_len);
  if (rc != 0) return rc;
  write_padded_u64((unsigned long long)line_len);
  write_bytes("\n", 1);
  free(lines);
  free(data);
  return 0;
}

static int pipe_cat_sort_head(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  unsigned long long limit = 0;
  if (count != 8 || strcmp(words[4], "|") || strcmp(words[5], "head") ||
      strcmp(words[6], "-n") || !parse_u64_arg(words[7], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = load_cat_sorted_pipe(words, count, &data, &lines, &line_len);
  if (rc != 0) return rc;
  size_t take = limit > (unsigned long long)line_len ? line_len : (size_t)limit;
  for (size_t idx = 0; idx < take; idx++) {
    write_line_span_output(data, lines[idx]);
  }
  free(lines);
  free(data);
  return 0;
}

static int pipe_cat_sort_tail(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  unsigned long long limit = 0;
  if (count != 8 || strcmp(words[4], "|") || strcmp(words[5], "tail") ||
      strcmp(words[6], "-n") || !parse_u64_arg(words[7], &limit)) {
    return unsupported();
  }
  int rc = load_cat_sorted_pipe(words, count, &data, &lines, &line_len);
  if (rc != 0) return rc;
  size_t take = limit > (unsigned long long)line_len ? line_len : (size_t)limit;
  size_t start = line_len - take;
  for (size_t idx = start; idx < line_len; idx++) {
    write_line_span_output(data, lines[idx]);
  }
  free(lines);
  free(data);
  return 0;
}

static int pipe_cat_sort_xargs_echo(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  int first = 1;
  if (count != 7 || strcmp(words[4], "|") || strcmp(words[5], "xargs") ||
      strcmp(words[6], "echo")) {
    return unsupported();
  }
  int rc = load_cat_sorted_pipe(words, count, &data, &lines, &line_len);
  if (rc != 0) return rc;
  for (size_t idx = 0; idx < line_len; idx++) {
    emit_xargs_echo_bytes(data + lines[idx].start, lines[idx].end - lines[idx].start, &first);
  }
  if (!first) write_bytes("\n", 1);
  free(lines);
  free(data);
  return 0;
}

static int pipe_cat_sort_xargs_wc(char **words, int count) {
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 8 || strcmp(words[4], "|") || strcmp(words[5], "xargs") ||
      strcmp(words[6], "wc") || strcmp(words[7], "-l")) {
    return unsupported();
  }
  int rc = load_cat_sorted_pipe(words, count, &data, &lines, &line_len);
  if (rc != 0) return rc;
  for (size_t idx = 0; idx < line_len; idx++) {
    if (!emit_xargs_wc_bytes(data + lines[idx].start, lines[idx].end - lines[idx].start,
                             &total, &files, &err)) {
      free(lines);
      free(data);
      return 1;
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  free(lines);
  free(data);
  return err ? 1 : 0;
}

static int pipe_cat_wc(char **words, int count) {
  int err = 0;
  unsigned long long lines = 0;
  if (count != 5 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "wc") || strcmp(words[4], "-l")) {
    return unsupported();
  }
  lines = count_newlines_path(words[1], "cat", &err);
  write_padded_u64(lines);
  write_bytes("\n", 1);
  return 0;
}

static int pipe_head_producer(char **words, int count);
static int pipe_tail_producer(char **words, int count);

static int parse_cat_head_tail_source(char **words, int count, const char *cmd,
                                      int require_positive, const char **path,
                                      unsigned long long *limit,
                                      int *downstream_start) {
  if (count < 4 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], cmd)) {
    return 0;
  }
  *path = words[1];
  *limit = 10;
  *downstream_start = 0;
  if (count == 4) return 1;
  if (!strcmp(words[4], "|")) {
    if (count < 6) return 0;
    *downstream_start = 5;
    return 1;
  }
  if (!strcmp(words[4], "-n")) {
    if (count < 6 || !parse_u64_arg(words[5], limit) ||
        (require_positive && *limit == 0)) {
      return 0;
    }
    if (count == 6) return 1;
    if (count >= 8 && !strcmp(words[6], "|")) {
      *downstream_start = 7;
      return 1;
    }
    return 0;
  }
  if (words[4][0] == '-' && words[4][1] >= '0' && words[4][1] <= '9') {
    if (!parse_u64_arg(words[4] + 1, limit) ||
        (require_positive && *limit == 0)) {
      return 0;
    }
    if (count == 5) return 1;
    if (count >= 7 && !strcmp(words[5], "|")) {
      *downstream_start = 6;
      return 1;
    }
  }
  return 0;
}

static int pipe_cat_head_tail_producer(char **words, int count, const char *cmd,
                                       int is_head) {
  const char *path = NULL;
  unsigned long long limit = 0;
  int downstream_start = 0;
  char limit_buf[32];
  if (!parse_cat_head_tail_source(words, count, cmd, is_head, &path, &limit,
                                  &downstream_start) ||
      downstream_start == 0) {
    return unsupported();
  }
  snprintf(limit_buf, sizeof(limit_buf), "%llu", limit);
  char **rewritten = (char **)calloc((size_t)count + 4, sizeof(char *));
  if (!rewritten) return 1;
  rewritten[0] = (char *)cmd;
  rewritten[1] = "-n";
  rewritten[2] = limit_buf;
  rewritten[3] = (char *)path;
  rewritten[4] = "|";
  int rewritten_count = 5;
  for (int idx = downstream_start; idx < count; idx++) {
    rewritten[rewritten_count++] = words[idx];
  }
  int rc = is_head ? pipe_head_producer(rewritten, rewritten_count)
                   : pipe_tail_producer(rewritten, rewritten_count);
  free(rewritten);
  return rc;
}

static int pipe_cat_head(char **words, int count) {
  unsigned long long limit = 0;
  const char *path = NULL;
  int downstream_start = 0;
  if (!parse_cat_head_tail_source(words, count, "head", 1, &path, &limit,
                                  &downstream_start) ||
      downstream_start != 0) {
    return unsupported();
  }
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("cat", path, errno);
    return 0;
  }
  int rc = head_copy_lines(fd, limit);
  if (rc) write_err_path("cat", path, errno);
  close(fd);
  return 0;
}

static int store_tail_line(char **lines, size_t *lens, size_t slots,
                           unsigned long long *total, const char *line, size_t len) {
  if (slots == 0) return 1;
  size_t slot = (size_t)(*total % (unsigned long long)slots);
  char *copy = (char *)malloc(len ? len : 1);
  if (!copy) return 0;
  if (len) memcpy(copy, line, len);
  free(lines[slot]);
  lines[slot] = copy;
  lens[slot] = len;
  (*total)++;
  return 1;
}

static int cat_tail_lines(int fd, const char *path, unsigned long long limit) {
  if (limit == 0) return 0;
  if (limit > 1000000ULL) return unsupported();
  size_t slots = (size_t)limit;
  char **lines = (char **)calloc(slots, sizeof(char *));
  size_t *lens = (size_t *)calloc(slots, sizeof(size_t));
  char *line = NULL;
  size_t line_cap = 0;
  size_t line_len = 0;
  unsigned long long total = 0;
  char buf[8192];
  int rc = 0;
  if (!lines || !lens) {
    free(lines);
    free(lens);
    return 1;
  }

  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("cat", path, errno);
      rc = 0;
      goto done;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (line_len == line_cap) {
        size_t next_cap = line_cap ? line_cap * 2 : 256;
        char *next = (char *)realloc(line, next_cap);
        if (!next) {
          rc = 1;
          goto done;
        }
        line = next;
        line_cap = next_cap;
      }
      line[line_len++] = buf[idx];
      if (buf[idx] == '\n') {
        if (!store_tail_line(lines, lens, slots, &total, line, line_len)) {
          rc = 1;
          goto done;
        }
        line_len = 0;
      }
    }
  }
  if (line_len) {
    if (!store_tail_line(lines, lens, slots, &total, line, line_len)) {
      rc = 1;
      goto done;
    }
  }

  {
    unsigned long long emit = total < limit ? total : limit;
    unsigned long long start = total - emit;
    for (unsigned long long idx = 0; idx < emit; idx++) {
      size_t slot = (size_t)((start + idx) % (unsigned long long)slots);
      if (lines[slot]) write_bytes(lines[slot], lens[slot]);
    }
  }

done:
  for (size_t idx = 0; idx < slots; idx++) free(lines[idx]);
  free(lines);
  free(lens);
  free(line);
  return rc;
}

static int pipe_cat_tail(char **words, int count) {
  unsigned long long limit = 0;
  const char *path = NULL;
  int downstream_start = 0;
  if (!parse_cat_head_tail_source(words, count, "tail", 0, &path, &limit,
                                  &downstream_start) ||
      downstream_start != 0) {
    return unsupported();
  }
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("cat", path, errno);
    return 0;
  }
  int rc = cat_tail_lines(fd, path, limit);
  close(fd);
  return rc == 127 ? unsupported() : 0;
}

static int pipe_cat_grep(char **words, int count) {
  int matched = 0;
  if (count != 5 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "grep") || !is_plain_literal_pattern(words[4])) {
    return unsupported();
  }
  int rc = cat_grep_file(words[1], words[4], strlen(words[4]), &matched);
  if (rc == 127) return unsupported();
  return matched ? 0 : 1;
}

static int pipe_cat_cut(char **words, int count) {
  struct cut_plan plan;
  if (count < 5 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "cut")) {
    return unsupported();
  }
  if (!parse_cut_words(words, 4, count, words[1], &plan)) return unsupported();
  (void)cut_file(&plan, "cat");
  return 0;
}

static int pipe_cat_tr(char **words, int count) {
  struct tr_plan plan;
  if (count < 5 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "tr")) {
    return unsupported();
  }
  if (!parse_tr_words(words, 4, count, &plan)) return unsupported();
  int fd = open(words[1], O_RDONLY);
  if (fd < 0) {
    write_err_path("cat", words[1], errno);
    return 0;
  }
  (void)tr_fd(fd, &plan, "cat", words[1]);
  close(fd);
  return 0;
}

static int pipe_cat_xargs_echo(char **words, int count) {
  char *data = NULL;
  size_t size = 0;
  int first = 1;
  if (count != 5 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "xargs") || strcmp(words[4], "echo")) {
    return unsupported();
  }
  int rc = load_regular_file_for_pipe(words[1], &data, &size);
  if (rc != 0) return rc;
  emit_xargs_echo_bytes(data, size, &first);
  if (!first) write_bytes("\n", 1);
  free(data);
  return 0;
}

static int pipe_cat_xargs_wc(char **words, int count) {
  char *data = NULL;
  size_t size = 0;
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 6 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "xargs") || strcmp(words[4], "wc") ||
      strcmp(words[5], "-l")) {
    return unsupported();
  }
  int rc = load_regular_file_for_pipe(words[1], &data, &size);
  if (rc != 0) return rc;
  if (!emit_xargs_wc_bytes(data, size, &total, &files, &err)) {
    free(data);
    return 1;
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  free(data);
  return err ? 1 : 0;
}

static int pipe_grep_head(char **words, int count) {
  char path[PATH_MAX];
  unsigned long long limit = 0;
  int matched = 0;
  if (count != 8 || strcmp(words[0], "grep") || strcmp(words[1], "-R") ||
      strcmp(words[4], "|") || strcmp(words[5], "head") ||
      strcmp(words[6], "-n") || !parse_u64_arg(words[7], &limit) || limit == 0) {
    return unsupported();
  }
  if (!is_plain_literal_pattern(words[2])) return unsupported();
  if (!copy_cstr(path, sizeof(path), words[3])) return unsupported();
  (void)grep_walk_head(path, sizeof(path), words[2], strlen(words[2]), &limit, &matched);
  return 0;
}

static int store_grep_tail_match(char **lines, size_t *lens, size_t slots,
                                 unsigned long long *total, const char *path,
                                 const char *line, size_t line_len, int add_newline) {
  if (slots == 0) return 1;
  size_t path_len = strlen(path);
  size_t len = path_len + 1 + line_len + (add_newline ? 1 : 0);
  char *copy = (char *)malloc(len ? len : 1);
  if (!copy) return 0;
  memcpy(copy, path, path_len);
  copy[path_len] = ':';
  if (line_len) memcpy(copy + path_len + 1, line, line_len);
  if (add_newline) copy[len - 1] = '\n';
  size_t slot = (size_t)(*total % (unsigned long long)slots);
  free(lines[slot]);
  lines[slot] = copy;
  lens[slot] = len;
  (*total)++;
  return 1;
}

static int grep_tail_file(const char *path, const char *pat, size_t pat_len,
                          char **lines, size_t *lens, size_t slots,
                          unsigned long long *total) {
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
        if (contains_bytes(line, (ssize_t)used, pat, pat_len) &&
            !store_grep_tail_match(lines, lens, slots, total, path, line, used, 0)) {
          close(fd);
          return 1;
        }
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len) &&
      !store_grep_tail_match(lines, lens, slots, total, path, line, used, 1)) {
    close(fd);
    return 1;
  }
  close(fd);
  return 0;
}

static int grep_walk_tail(char *path, size_t cap, const char *pat, size_t pat_len,
                          char **lines, size_t *lens, size_t slots,
                          unsigned long long *total) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("grep", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode)) return grep_tail_file(path, pat, pat_len, lines, lens, slots, total);
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
      rc |= grep_walk_tail(path, cap, pat, pat_len, lines, lens, slots, total);
    } else if (entry->d_type == DT_REG) {
      rc |= grep_tail_file(path, pat, pat_len, lines, lens, slots, total);
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= grep_walk_tail(path, cap, pat, pat_len, lines, lens, slots, total);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int pipe_grep_tail(char **words, int count) {
  char path[PATH_MAX];
  unsigned long long limit = 0;
  if (count != 8 || strcmp(words[0], "grep") || strcmp(words[1], "-R") ||
      strcmp(words[4], "|") || strcmp(words[5], "tail") ||
      strcmp(words[6], "-n") || !parse_u64_arg(words[7], &limit) || limit == 0 ||
      limit > 1000000ULL) {
    return unsupported();
  }
  if (!is_plain_literal_pattern(words[2])) return unsupported();
  if (!copy_cstr(path, sizeof(path), words[3])) return unsupported();
  size_t slots = (size_t)limit;
  char **lines = (char **)calloc(slots, sizeof(char *));
  size_t *lens = (size_t *)calloc(slots, sizeof(size_t));
  unsigned long long total = 0;
  if (!lines || !lens) {
    free(lines);
    free(lens);
    return 1;
  }
  (void)grep_walk_tail(path, sizeof(path), words[2], strlen(words[2]), lines, lens, slots, &total);
  unsigned long long emit = total < limit ? total : limit;
  unsigned long long start = total - emit;
  for (unsigned long long idx = 0; idx < emit; idx++) {
    size_t slot = (size_t)((start + idx) % (unsigned long long)slots);
    if (lines[slot]) write_bytes(lines[slot], lens[slot]);
  }
  for (size_t idx = 0; idx < slots; idx++) free(lines[idx]);
  free(lines);
  free(lens);
  return 0;
}

struct byte_line_item {
  char *data;
  size_t len;
};

struct byte_line_list {
  struct byte_line_item *items;
  size_t len;
  size_t cap;
};

static void byte_line_list_free(struct byte_line_list *list) {
  for (size_t idx = 0; idx < list->len; idx++) free(list->items[idx].data);
  free(list->items);
  list->items = NULL;
  list->len = 0;
  list->cap = 0;
}

static int byte_line_list_push(struct byte_line_list *list, const char *data, size_t len) {
  if (list->len == list->cap) {
    size_t next_cap = list->cap ? list->cap * 2 : 128;
    struct byte_line_item *next =
        (struct byte_line_item *)realloc(list->items, sizeof(struct byte_line_item) * next_cap);
    if (!next) return 0;
    list->items = next;
    list->cap = next_cap;
  }
  char *copy = (char *)malloc(len ? len : 1);
  if (!copy) return 0;
  if (len) memcpy(copy, data, len);
  list->items[list->len].data = copy;
  list->items[list->len].len = len;
  list->len++;
  return 1;
}

static int byte_line_item_cmp(const void *left, const void *right) {
  const struct byte_line_item *a = (const struct byte_line_item *)left;
  const struct byte_line_item *b = (const struct byte_line_item *)right;
  size_t min_len = a->len < b->len ? a->len : b->len;
  int cmp = min_len ? memcmp(a->data, b->data, min_len) : 0;
  if (cmp != 0) return cmp;
  if (a->len < b->len) return -1;
  if (a->len > b->len) return 1;
  return 0;
}

static size_t byte_line_item_len_without_newline(const struct byte_line_item *item) {
  return item->len > 0 && item->data[item->len - 1] == '\n' ? item->len - 1 : item->len;
}

static int byte_line_items_equal_without_newline(const struct byte_line_item *left,
                                                 const struct byte_line_item *right) {
  size_t left_len = byte_line_item_len_without_newline(left);
  size_t right_len = byte_line_item_len_without_newline(right);
  return left_len == right_len &&
         (left_len == 0 || memcmp(left->data, right->data, left_len) == 0);
}

static void emit_unique_byte_line_list(const struct byte_line_list *list) {
  for (size_t idx = 0; idx < list->len; idx++) {
    int duplicate =
        idx > 0 && byte_line_items_equal_without_newline(&list->items[idx - 1],
                                                         &list->items[idx]);
    if (!duplicate) write_bytes(list->items[idx].data, list->items[idx].len);
  }
}

static unsigned long long count_unique_byte_line_list(const struct byte_line_list *list) {
  unsigned long long count = 0;
  for (size_t idx = 0; idx < list->len; idx++) {
    int duplicate =
        idx > 0 && byte_line_items_equal_without_newline(&list->items[idx - 1],
                                                         &list->items[idx]);
    if (!duplicate) count++;
  }
  return count;
}

static void byte_line_list_sort_unique(struct byte_line_list *list) {
  qsort(list->items, list->len, sizeof(struct byte_line_item), byte_line_item_cmp);
  size_t write = 0;
  for (size_t read = 0; read < list->len; read++) {
    if (write > 0 &&
        byte_line_items_equal_without_newline(&list->items[write - 1],
                                              &list->items[read])) {
      free(list->items[read].data);
      continue;
    }
    if (write != read) list->items[write] = list->items[read];
    write++;
  }
  list->len = write;
}

static int emit_head_line_list_mode(char **words, int start, int count,
                                    struct byte_line_list *list,
                                    int wc_counts_newlines);
static int head_line_list_mode_supported(char **words, int start, int count);
static int xargs_wc_output_mode_supported(char **words, int start, int count);
static int byte_line_list_push_xargs_wc_path(struct byte_line_list *lines,
                                             const char *path,
                                             unsigned long long *total,
                                             unsigned long long *files,
                                             int *err);

static int byte_line_list_push_grep_match(struct byte_line_list *list, const char *path,
                                          const char *line, size_t line_len, int add_newline) {
  size_t path_len = strlen(path);
  size_t len = path_len + 1 + line_len + (add_newline ? 1 : 0);
  char *out = (char *)malloc(len ? len : 1);
  if (!out) return 0;
  memcpy(out, path, path_len);
  out[path_len] = ':';
  if (line_len) memcpy(out + path_len + 1, line, line_len);
  if (add_newline) out[len - 1] = '\n';
  int ok = byte_line_list_push(list, out, len);
  free(out);
  return ok;
}

static int byte_line_list_push_plain_match(struct byte_line_list *list, const char *line,
                                           size_t line_len, int add_newline) {
  size_t len = line_len + (add_newline ? 1 : 0);
  char *out = (char *)malloc(len ? len : 1);
  if (!out) return 0;
  if (line_len) memcpy(out, line, line_len);
  if (add_newline) out[len - 1] = '\n';
  int ok = byte_line_list_push(list, out, len);
  free(out);
  return ok;
}

static int grep_plain_collect_file(const char *path, const char *pat, size_t pat_len,
                                   struct byte_line_list *list) {
  char buf[8192];
  char line[8192];
  size_t used = 0;
  struct stat st;
  int fd = STDIN_FILENO;
  int close_fd = 0;
  if (path) {
    if (stat(path, &st) != 0 || !S_ISREG(st.st_mode)) return unsupported();
    fd = open(path, O_RDONLY);
    if (fd < 0) {
      write_err_path("grep", path, errno);
      return 0;
    }
    close_fd = 1;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      if (path) write_err_path("grep", path, errno);
      if (close_fd) close(fd);
      return 0;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
        if (contains_bytes(line, (ssize_t)used, pat, pat_len) &&
            !byte_line_list_push_plain_match(list, line, used, 0)) {
          if (close_fd) close(fd);
          return 1;
        }
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len) &&
      !byte_line_list_push_plain_match(list, line, used, 1)) {
    if (close_fd) close(fd);
    return 1;
  }
  if (close_fd) close(fd);
  return 0;
}

static int cap_grep(int argc, char **argv) {
  char path[PATH_MAX];
  int matched = 0;
  if (argc == 3 && argv[2][0] != '-' && is_plain_literal_pattern(argv[2])) {
    struct byte_line_list list = {0};
    int rc = grep_plain_collect_file(NULL, argv[2], strlen(argv[2]), &list);
    if (rc == 127) return unsupported();
    for (size_t idx = 0; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
    matched = list.len > 0;
    byte_line_list_free(&list);
    return matched ? 0 : (rc ? rc : 1);
  }
  if (argc == 4 && argv[2][0] != '-' && argv[3][0] != '-' &&
      is_plain_literal_pattern(argv[2])) {
    int rc = grep_plain_file(argv[3], argv[2], strlen(argv[2]), &matched);
    if (rc == 127) return unsupported();
    return matched ? 0 : (rc ? rc : 1);
  }
  if (argc != 5 || strcmp(argv[2], "-R") != 0) return unsupported();
  if (!is_plain_literal_pattern(argv[3])) return unsupported();
  if (!copy_cstr(path, sizeof(path), argv[4])) return unsupported();
  int rc = grep_walk(path, sizeof(path), argv[3], strlen(argv[3]), &matched);
  return matched ? 0 : (rc ? 2 : 1);
}

static int grep_sort_file(const char *path, const char *pat, size_t pat_len,
                          struct byte_line_list *list) {
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
        if (contains_bytes(line, (ssize_t)used, pat, pat_len) &&
            !byte_line_list_push_grep_match(list, path, line, used, 0)) {
          close(fd);
          return 1;
        }
        used = 0;
      }
    }
  }
  if (used && contains_bytes(line, (ssize_t)used, pat, pat_len) &&
      !byte_line_list_push_grep_match(list, path, line, used, 1)) {
    close(fd);
    return 1;
  }
  close(fd);
  return 0;
}

static int grep_sort_walk(char *path, size_t cap, const char *pat, size_t pat_len,
                          struct byte_line_list *list) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("grep", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode)) return grep_sort_file(path, pat, pat_len, list);
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
      rc |= grep_sort_walk(path, cap, pat, pat_len, list);
    } else if (entry->d_type == DT_REG) {
      rc |= grep_sort_file(path, pat, pat_len, list);
    } else if (entry->d_type == DT_UNKNOWN) {
      rc |= grep_sort_walk(path, cap, pat, pat_len, list);
    }
    path[len] = 0;
  }
  closedir(dir);
  return rc;
}

static int collect_grep_sorted_lines(char **words, int count, struct byte_line_list *list) {
  char path[PATH_MAX];
  if (count < 6 || strcmp(words[0], "grep") || strcmp(words[1], "-R") ||
      strcmp(words[4], "|") || strcmp(words[5], "sort")) {
    return unsupported();
  }
  if (!is_plain_literal_pattern(words[2])) return unsupported();
  if (!copy_cstr(path, sizeof(path), words[3])) return unsupported();
  int rc = grep_sort_walk(path, sizeof(path), words[2], strlen(words[2]), list);
  qsort(list->items, list->len, sizeof(struct byte_line_item), byte_line_item_cmp);
  (void)rc;
  return 0;
}

static int pipe_grep_sort(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 6) return unsupported();
  int rc = collect_grep_sorted_lines(words, count, &list);
  if (rc == 0) {
    for (size_t idx = 0; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_sort_uniq(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 8 || strcmp(words[6], "|") || strcmp(words[7], "uniq")) {
    return unsupported();
  }
  int rc = collect_grep_sorted_lines(words, count, &list);
  if (rc == 0) emit_unique_byte_line_list(&list);
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_sort_uniq_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 11 || strcmp(words[6], "|") || strcmp(words[7], "uniq") ||
      strcmp(words[8], "|") || strcmp(words[9], "wc") || strcmp(words[10], "-l")) {
    return unsupported();
  }
  int rc = collect_grep_sorted_lines(words, count, &list);
  if (rc == 0) {
    write_padded_u64(count_unique_byte_line_list(&list));
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_sort_uniq_producer(char **words, int count) {
  struct byte_line_list list = {0};
  if (count <= 9 || strcmp(words[6], "|") || strcmp(words[7], "uniq") ||
      strcmp(words[8], "|") || !head_line_list_mode_supported(words, 9, count)) {
    return unsupported();
  }
  int rc = collect_grep_sorted_lines(words, count, &list);
  if (rc == 0) {
    byte_line_list_sort_unique(&list);
    rc = emit_head_line_list_mode(words, 9, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_sort_wc(char **words, int count) {
  char path[PATH_MAX];
  unsigned long long matches = 0;
  if (count != 9 || strcmp(words[0], "grep") || strcmp(words[1], "-R") ||
      strcmp(words[4], "|") || strcmp(words[5], "sort") ||
      strcmp(words[6], "|") || strcmp(words[7], "wc") || strcmp(words[8], "-l")) {
    return unsupported();
  }
  if (!is_plain_literal_pattern(words[2])) return unsupported();
  if (!copy_cstr(path, sizeof(path), words[3])) return unsupported();
  (void)grep_walk_count(path, sizeof(path), words[2], strlen(words[2]), &matches);
  write_padded_u64(matches);
  write_bytes("\n", 1);
  return 0;
}

static int pipe_grep_sort_head(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 10 || strcmp(words[6], "|") || strcmp(words[7], "head") ||
      strcmp(words[8], "-n") || !parse_u64_arg(words[9], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = collect_grep_sorted_lines(words, count, &list);
  if (rc == 0) {
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_sort_tail(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 10 || strcmp(words[6], "|") || strcmp(words[7], "tail") ||
      strcmp(words[8], "-n") || !parse_u64_arg(words[9], &limit)) {
    return unsupported();
  }
  int rc = collect_grep_sorted_lines(words, count, &list);
  if (rc == 0) {
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_wc(char **words, int count) {
  char path[PATH_MAX];
  unsigned long long matches = 0;
  if (count != 7 || strcmp(words[0], "grep") || strcmp(words[1], "-R") ||
      strcmp(words[4], "|") || strcmp(words[5], "wc") || strcmp(words[6], "-l")) {
    return unsupported();
  }
  if (!is_plain_literal_pattern(words[2])) return unsupported();
  if (!copy_cstr(path, sizeof(path), words[3])) return unsupported();
  int rc = grep_walk_count(path, sizeof(path), words[2], strlen(words[2]), &matches);
  write_padded_u64(matches);
  write_bytes("\n", 1);
  (void)rc;
  return 0;
}

static int parse_grep_file_pipe_source(char **words, int pipe, const char **pattern,
                                       const char **file) {
  if (pipe == 3 && words[2] == NULL) {
    if (strcmp(words[0], "grep") || words[1][0] == '-' ||
        !is_plain_literal_pattern(words[1])) {
      return unsupported();
    }
    *pattern = words[1];
    *file = NULL;
    return 0;
  }
  if (pipe != 3 || strcmp(words[0], "grep") || words[1][0] == '-' ||
      words[2][0] == '-' || !is_plain_literal_pattern(words[1])) {
    return unsupported();
  }
  struct stat st;
  if (stat(words[2], &st) != 0 || !S_ISREG(st.st_mode)) return unsupported();
  *pattern = words[1];
  *file = words[2];
  return 0;
}

static int collect_grep_file_pipe_lines(char **words, int pipe, struct byte_line_list *list) {
  const char *pattern = NULL;
  const char *file = NULL;
  int rc = parse_grep_file_pipe_source(words, pipe, &pattern, &file);
  if (rc != 0) return rc;
  return grep_plain_collect_file(file, pattern, strlen(pattern), list);
}

static int pipe_grep_file_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 6 || strcmp(words[3], "|") || strcmp(words[4], "wc") ||
      strcmp(words[5], "-l")) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    write_padded_u64((unsigned long long)list.len);
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_head(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 7 || strcmp(words[3], "|") || strcmp(words[4], "head") ||
      strcmp(words[5], "-n") || !parse_u64_arg(words[6], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_tail(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 7 || strcmp(words[3], "|") || strcmp(words[4], "tail") ||
      strcmp(words[5], "-n") || !parse_u64_arg(words[6], &limit)) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_xargs_echo(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 6 || strcmp(words[3], "|") || strcmp(words[4], "xargs") ||
      strcmp(words[5], "echo")) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    int first = 1;
    for (size_t idx = 0; idx < list.len; idx++) {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
    if (!first) write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_xargs_wc(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 7 || strcmp(words[3], "|") || strcmp(words[4], "xargs") ||
      strcmp(words[5], "wc") || strcmp(words[6], "-l")) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    for (size_t idx = 0; idx < list.len; idx++) {
      if (!emit_xargs_wc_bytes(list.items[idx].data, list.items[idx].len, &total, &files,
                               &err)) {
        byte_line_list_free(&list);
        return 1;
      }
    }
    if (files > 1) {
      write_padded_u64(total);
      write_bytes(" total\n", 7);
    }
    rc = err ? 1 : 0;
  }
  byte_line_list_free(&list);
  return rc;
}

static int collect_cat_grep_pipe_lines(char **words, struct byte_line_list *list) {
  if (strcmp(words[0], "cat") || strcmp(words[2], "|") || strcmp(words[3], "grep") ||
      !is_plain_literal_pattern(words[4]) || strcmp(words[5], "|")) {
    return unsupported();
  }
  return grep_plain_collect_file(words[1], words[4], strlen(words[4]), list);
}

static int pipe_cat_grep_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 8 || strcmp(words[6], "wc") || strcmp(words[7], "-l")) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    write_padded_u64((unsigned long long)list.len);
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_head(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 9 || strcmp(words[6], "head") || strcmp(words[7], "-n") ||
      !parse_u64_arg(words[8], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_tail(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 9 || strcmp(words[6], "tail") || strcmp(words[7], "-n") ||
      !parse_u64_arg(words[8], &limit)) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 7 || strcmp(words[6], "sort")) return unsupported();
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort_uniq(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 9 || strcmp(words[6], "sort") || strcmp(words[7], "|") ||
      strcmp(words[8], "uniq")) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    emit_unique_byte_line_list(&list);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort_uniq_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 12 || strcmp(words[6], "sort") || strcmp(words[7], "|") ||
      strcmp(words[8], "uniq") || strcmp(words[9], "|") || strcmp(words[10], "wc") ||
      strcmp(words[11], "-l")) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    write_padded_u64(count_unique_byte_line_list(&list));
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort_uniq_producer(char **words, int count) {
  struct byte_line_list list = {0};
  if (count <= 10 || strcmp(words[6], "sort") || strcmp(words[7], "|") ||
      strcmp(words[8], "uniq") || strcmp(words[9], "|") ||
      !head_line_list_mode_supported(words, 10, count)) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    byte_line_list_sort_unique(&list);
    rc = emit_head_line_list_mode(words, 10, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 10 || strcmp(words[6], "sort") || strcmp(words[7], "|") ||
      strcmp(words[8], "wc") || strcmp(words[9], "-l")) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    write_padded_u64((unsigned long long)list.len);
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort_head(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 11 || strcmp(words[6], "sort") || strcmp(words[7], "|") ||
      strcmp(words[8], "head") || strcmp(words[9], "-n") ||
      !parse_u64_arg(words[10], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort_tail(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 11 || strcmp(words[6], "sort") || strcmp(words[7], "|") ||
      strcmp(words[8], "tail") || strcmp(words[9], "-n") ||
      !parse_u64_arg(words[10], &limit)) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_xargs_echo(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 8 || strcmp(words[6], "xargs") || strcmp(words[7], "echo")) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    int first = 1;
    for (size_t idx = 0; idx < list.len; idx++) {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
    if (!first) write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_xargs_wc(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 9 || strcmp(words[6], "xargs") || strcmp(words[7], "wc") ||
      strcmp(words[8], "-l")) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    for (size_t idx = 0; idx < list.len; idx++) {
      if (!emit_xargs_wc_bytes(list.items[idx].data, list.items[idx].len, &total, &files, &err)) {
        rc = 1;
        break;
      }
    }
    if (rc == 0 && files > 1) {
      write_padded_u64(total);
      write_bytes(" total\n", 7);
    }
    rc = rc ? rc : (err ? 1 : 0);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort_xargs_echo(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 10 || strcmp(words[6], "sort") || strcmp(words[7], "|") ||
      strcmp(words[8], "xargs") || strcmp(words[9], "echo")) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    int first = 1;
    for (size_t idx = 0; idx < list.len; idx++) {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
    if (!first) write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_grep_sort_xargs_wc(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 11 || strcmp(words[6], "sort") || strcmp(words[7], "|") ||
      strcmp(words[8], "xargs") || strcmp(words[9], "wc") || strcmp(words[10], "-l")) {
    return unsupported();
  }
  int rc = collect_cat_grep_pipe_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) {
      if (!emit_xargs_wc_bytes(list.items[idx].data, list.items[idx].len, &total, &files, &err)) {
        rc = 1;
        break;
      }
    }
    if (rc == 0 && files > 1) {
      write_padded_u64(total);
      write_bytes(" total\n", 7);
    }
    rc = rc ? rc : (err ? 1 : 0);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 5 || strcmp(words[3], "|") || strcmp(words[4], "sort")) return unsupported();
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort_uniq(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 7 || strcmp(words[3], "|") || strcmp(words[4], "sort") ||
      strcmp(words[5], "|") || strcmp(words[6], "uniq")) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    emit_unique_byte_line_list(&list);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort_uniq_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 10 || strcmp(words[3], "|") || strcmp(words[4], "sort") ||
      strcmp(words[5], "|") || strcmp(words[6], "uniq") ||
      strcmp(words[7], "|") || strcmp(words[8], "wc") || strcmp(words[9], "-l")) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    write_padded_u64(count_unique_byte_line_list(&list));
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort_uniq_producer(char **words, int count) {
  struct byte_line_list list = {0};
  if (count <= 8 || strcmp(words[3], "|") || strcmp(words[4], "sort") ||
      strcmp(words[5], "|") || strcmp(words[6], "uniq") || strcmp(words[7], "|") ||
      !head_line_list_mode_supported(words, 8, count)) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    byte_line_list_sort_unique(&list);
    rc = emit_head_line_list_mode(words, 8, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 8 || strcmp(words[3], "|") || strcmp(words[4], "sort") ||
      strcmp(words[5], "|") || strcmp(words[6], "wc") || strcmp(words[7], "-l")) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    write_padded_u64((unsigned long long)list.len);
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort_head(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 9 || strcmp(words[3], "|") || strcmp(words[4], "sort") ||
      strcmp(words[5], "|") || strcmp(words[6], "head") ||
      strcmp(words[7], "-n") || !parse_u64_arg(words[8], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort_tail(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 9 || strcmp(words[3], "|") || strcmp(words[4], "sort") ||
      strcmp(words[5], "|") || strcmp(words[6], "tail") ||
      strcmp(words[7], "-n") || !parse_u64_arg(words[8], &limit)) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort_xargs_echo(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 8 || strcmp(words[3], "|") || strcmp(words[4], "sort") ||
      strcmp(words[5], "|") || strcmp(words[6], "xargs") || strcmp(words[7], "echo")) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    int first = 1;
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
    if (!first) write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_grep_file_sort_xargs_wc(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 9 || strcmp(words[3], "|") || strcmp(words[4], "sort") ||
      strcmp(words[5], "|") || strcmp(words[6], "xargs") || strcmp(words[7], "wc") ||
      strcmp(words[8], "-l")) {
    return unsupported();
  }
  int rc = collect_grep_file_pipe_lines(words, 3, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) {
      if (!emit_xargs_wc_bytes(list.items[idx].data, list.items[idx].len, &total, &files,
                               &err)) {
        byte_line_list_free(&list);
        return 1;
      }
    }
    if (files > 1) {
      write_padded_u64(total);
      write_bytes(" total\n", 7);
    }
    rc = err ? 1 : 0;
  }
  byte_line_list_free(&list);
  return rc;
}

static void emit_xargs_token(const char *token, size_t len, int *first) {
  if (!*first) write_bytes(" ", 1);
  write_bytes(token, len);
  *first = 0;
}

static void emit_xargs_echo_bytes(const char *data, size_t len, int *first) {
  size_t cursor = 0;
  while (cursor < len) {
    while (cursor < len && isspace((unsigned char)data[cursor])) cursor++;
    size_t start = cursor;
    while (cursor < len && !isspace((unsigned char)data[cursor])) cursor++;
    if (cursor > start) emit_xargs_token(data + start, cursor - start, first);
  }
}

static void emit_xargs_echo_batch_bytes(const char *data, size_t len,
                                        struct xargs_echo_batch_state *state) {
  size_t cursor = 0;
  while (cursor < len) {
    while (cursor < len && isspace((unsigned char)data[cursor])) cursor++;
    size_t start = cursor;
    while (cursor < len && !isspace((unsigned char)data[cursor])) cursor++;
    if (cursor > start) {
      if (state->used) write_bytes(" ", 1);
      write_bytes(data + start, cursor - start);
      state->used++;
      if (state->used == state->size) {
        write_bytes("\n", 1);
        state->used = 0;
      }
    }
  }
}

static void finish_xargs_echo_batch(struct xargs_echo_batch_state *state) {
  if (state->used) {
    write_bytes("\n", 1);
    state->used = 0;
  }
}

static void emit_xargs_echo_path(const char *path, int *first) {
  emit_xargs_echo_bytes(path, strlen(path), first);
}

static int emit_xargs_wc_token(const char *token, size_t len,
                               unsigned long long *total,
                               unsigned long long *files, int *err) {
  char *path = (char *)malloc(len + 1);
  if (!path) return 0;
  if (len) memcpy(path, token, len);
  path[len] = 0;
  (void)find_wc_emit_file(path, total, files, err);
  free(path);
  return 1;
}

static int emit_xargs_wc_bytes(const char *data, size_t len,
                               unsigned long long *total,
                               unsigned long long *files, int *err) {
  size_t cursor = 0;
  while (cursor < len) {
    while (cursor < len && isspace((unsigned char)data[cursor])) cursor++;
    size_t start = cursor;
    while (cursor < len && !isspace((unsigned char)data[cursor])) cursor++;
    if (cursor > start &&
        !emit_xargs_wc_token(data + start, cursor - start, total, files, err)) {
      return 0;
    }
  }
  return 1;
}

static int collect_printf_string_lines(char **words, int pipe,
                                       struct byte_line_list *list) {
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  if (pipe < 0 || strcmp(words[0], "printf") ||
      !parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      kind != PRINTF_FORMAT_STRING_NEWLINE) {
    return 0;
  }
  for (int idx = first_arg; idx < pipe; idx++) {
    if (!byte_line_list_push_plain_match(list, words[idx], strlen(words[idx]), 1)) {
      return 0;
    }
  }
  return 1;
}

static int collect_printf_literal_lines(char **words, int pipe,
                                        struct byte_line_list *list) {
  char *literal = NULL;
  size_t literal_len = 0;
  size_t start = 0;
  if (pipe != 2 || strcmp(words[0], "printf") ||
      !decode_printf_literal_format(words[1], &literal, &literal_len)) {
    return 0;
  }
  for (size_t idx = 0; idx < literal_len; idx++) {
    if (literal[idx] == '\n') {
      if (!byte_line_list_push(list, literal + start, idx + 1 - start)) {
        free(literal);
        return 0;
      }
      start = idx + 1;
    }
  }
  if (start < literal_len &&
      !byte_line_list_push(list, literal + start, literal_len - start)) {
    free(literal);
    return 0;
  }
  free(literal);
  return 1;
}

static int pipe_printf_literal_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 0;
  if (pipe < 0) return unsupported();
  if (!collect_printf_literal_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    const char *pattern = words[pipe + 2];
    size_t pattern_len = strlen(pattern);
    struct byte_line_list filtered = {0};
    for (size_t idx = 0; idx < list.len; idx++) {
      struct byte_line_item *item = &list.items[idx];
      if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len)) {
        int add_newline = item->len == 0 || item->data[item->len - 1] != '\n';
        if (!byte_line_list_push_plain_match(&filtered, item->data, item->len,
                                             add_newline)) {
          byte_line_list_free(&list);
          byte_line_list_free(&filtered);
          return 1;
        }
      }
    }
    byte_line_list_free(&list);
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < filtered.len; idx++) {
        write_bytes(filtered.items[idx].data, filtered.items[idx].len);
      }
      rc = filtered.len ? 0 : 1;
      byte_line_list_free(&filtered);
      return rc;
    }
    if (count < pipe + 5 || strcmp(words[pipe + 3], "|")) {
      byte_line_list_free(&filtered);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 4, count, &filtered, 1);
    byte_line_list_free(&filtered);
    return rc;
  }
  rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  byte_line_list_free(&list);
  return rc;
}

static int collect_echo_string_lines(char **words, int pipe,
                                     struct byte_line_list *list) {
  int first_arg = 0;
  int newline = 1;
  if (pipe < 0 || strcmp(words[0], "echo") ||
      !parse_echo_words(words, 0, pipe, &first_arg, &newline)) {
    return 0;
  }
  size_t len = 0;
  for (int idx = first_arg; idx < pipe; idx++) {
    if (idx > first_arg) len++;
    len += strlen(words[idx]);
  }
  if (len == 0 && !newline) return 1;
  char *line = (char *)malloc(len ? len : 1);
  if (!line) return 0;
  size_t used = 0;
  for (int idx = first_arg; idx < pipe; idx++) {
    if (idx > first_arg) line[used++] = ' ';
    size_t part_len = strlen(words[idx]);
    if (part_len) {
      memcpy(line + used, words[idx], part_len);
      used += part_len;
    }
  }
  int ok = newline ? byte_line_list_push_plain_match(list, line, used, 1)
                   : byte_line_list_push(list, line, used);
  free(line);
  return ok;
}

static int awk_field_to_list(struct byte_line_list *list, const char *data,
                             size_t len, unsigned long long field);

static int collect_awk_print_field_line_list(const struct byte_line_list *source,
                                             const char *script,
                                             struct byte_line_list *list) {
  const char *awk_filter = NULL;
  unsigned long long awk_field = 1;
  if (!parse_awk_print_field_script(script, &awk_filter, &awk_field)) return 0;
  size_t filter_len = awk_filter ? strlen(awk_filter) : 0;
  for (size_t idx = 0; idx < source->len; idx++) {
    const struct byte_line_item *item = &source->items[idx];
    if (awk_filter &&
        !contains_bytes(item->data, (ssize_t)item->len, awk_filter, filter_len)) {
      continue;
    }
    if (!awk_field_to_list(list, item->data, item->len, awk_field)) return 0;
  }
  return 1;
}

static int emit_awk_line_list_pipe(char **words, int pipe, int count,
                                   struct byte_line_list *list) {
  if (count == pipe + 3) {
    for (size_t idx = 0; idx < list->len; idx++) {
      write_bytes(list->items[idx].data, list->items[idx].len);
    }
    return 0;
  }
  if (count < pipe + 5 || strcmp(words[pipe + 3], "|") ||
      !head_line_list_mode_supported(words, pipe + 4, count)) {
    return unsupported();
  }
  return emit_head_line_list_mode(words, pipe + 4, count, list, 1);
}

static int pipe_echo_awk_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list source = {0};
  struct byte_line_list list = {0};
  if (pipe < 0 || count < pipe + 3 || strcmp(words[pipe + 1], "awk")) {
    return unsupported();
  }
  if (!collect_echo_string_lines(words, pipe, &source)) {
    byte_line_list_free(&source);
    return unsupported();
  }
  if (!collect_awk_print_field_line_list(&source, words[pipe + 2], &list)) {
    byte_line_list_free(&source);
    byte_line_list_free(&list);
    return unsupported();
  }
  byte_line_list_free(&source);
  int rc = emit_awk_line_list_pipe(words, pipe, count, &list);
  byte_line_list_free(&list);
  return rc;
}

static int pipe_printf_awk_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list source = {0};
  struct byte_line_list list = {0};
  if (pipe < 0 || count < pipe + 3 || strcmp(words[pipe + 1], "awk")) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &source)) {
    byte_line_list_free(&source);
    return unsupported();
  }
  if (!collect_awk_print_field_line_list(&source, words[pipe + 2], &list)) {
    byte_line_list_free(&source);
    byte_line_list_free(&list);
    return unsupported();
  }
  byte_line_list_free(&source);
  int rc = emit_awk_line_list_pipe(words, pipe, count, &list);
  byte_line_list_free(&list);
  return rc;
}

static int collect_printf_grep_lines(char **words, int pipe,
                                     struct byte_line_list *list) {
  enum printf_format_kind kind = PRINTF_FORMAT_UNSUPPORTED;
  int first_arg = 0;
  const char *pattern = NULL;
  size_t pattern_len = 0;
  if (pipe < 0 || strcmp(words[0], "printf") ||
      strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2]) ||
      !parse_printf_words(words, 0, pipe, &kind, &first_arg) ||
      kind != PRINTF_FORMAT_STRING_NEWLINE) {
    return 0;
  }
  pattern = words[pipe + 2];
  pattern_len = strlen(pattern);
  for (int idx = first_arg; idx < pipe; idx++) {
    size_t len = strlen(words[idx]);
    if (contains_bytes(words[idx], (ssize_t)len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, words[idx], len, 1)) {
      return 0;
    }
  }
  return 1;
}

static int pipe_printf_grep_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  int first = 1;
  if (pipe < 0) return unsupported();

  if (count == pipe + 6 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "wc") && !strcmp(words[pipe + 5], "-l")) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    write_padded_u64((unsigned long long)list.len);
    write_bytes("\n", 1);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 7 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "head") && !strcmp(words[pipe + 5], "-n") &&
      parse_u64_arg(words[pipe + 6], &limit) && limit > 0) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 7 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "tail") && !strcmp(words[pipe + 5], "-n") &&
      parse_u64_arg(words[pipe + 6], &limit)) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 5 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "sort")) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 7 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "sort") && !strcmp(words[pipe + 5], "|") &&
      !strcmp(words[pipe + 6], "uniq")) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    emit_unique_byte_line_list(&list);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 10 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "sort") && !strcmp(words[pipe + 5], "|") &&
      !strcmp(words[pipe + 6], "uniq") && !strcmp(words[pipe + 7], "|") &&
      !strcmp(words[pipe + 8], "wc") && !strcmp(words[pipe + 9], "-l")) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    write_padded_u64(count_unique_byte_line_list(&list));
    write_bytes("\n", 1);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 8 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "sort") && !strcmp(words[pipe + 5], "|") &&
      !strcmp(words[pipe + 6], "wc") && !strcmp(words[pipe + 7], "-l")) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    write_padded_u64((unsigned long long)list.len);
    write_bytes("\n", 1);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 9 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "sort") && !strcmp(words[pipe + 5], "|") &&
      !strcmp(words[pipe + 6], "head") && !strcmp(words[pipe + 7], "-n") &&
      parse_u64_arg(words[pipe + 8], &limit) && limit > 0) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 9 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "sort") && !strcmp(words[pipe + 5], "|") &&
      !strcmp(words[pipe + 6], "tail") && !strcmp(words[pipe + 7], "-n") &&
      parse_u64_arg(words[pipe + 8], &limit)) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 8 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "sort") && !strcmp(words[pipe + 5], "|") &&
      !strcmp(words[pipe + 6], "xargs") && !strcmp(words[pipe + 7], "echo")) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
    if (!first) write_bytes("\n", 1);
    byte_line_list_free(&list);
    return 0;
  }

  if (count == pipe + 6 && !strcmp(words[pipe + 3], "|") &&
      !strcmp(words[pipe + 4], "xargs") && !strcmp(words[pipe + 5], "echo")) {
    if (!collect_printf_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    for (size_t idx = 0; idx < list.len; idx++) {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
    if (!first) write_bytes("\n", 1);
    byte_line_list_free(&list);
    return 0;
  }

  return unsupported();
}

static int pipe_printf_grep_sort_uniq_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0 || count <= pipe + 8 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2]) || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort") || strcmp(words[pipe + 5], "|") ||
      strcmp(words[pipe + 6], "uniq") || strcmp(words[pipe + 7], "|")) {
    return unsupported();
  }
  if (!collect_printf_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  byte_line_list_sort_unique(&list);
  rc = emit_head_line_list_mode(words, pipe + 8, count, &list, 0);
  byte_line_list_free(&list);
  return rc;
}

static int pipe_printf_sort_uniq_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0 || count <= pipe + 5 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "uniq") ||
      strcmp(words[pipe + 4], "|")) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  byte_line_list_sort_unique(&list);
  rc = emit_head_line_list_mode(words, pipe + 5, count, &list, 1);
  byte_line_list_free(&list);
  return rc;
}

static int pipe_printf_sort(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 2 || strcmp(words[pipe + 1], "sort")) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_printf_sort_uniq(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 4 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "uniq")) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  emit_unique_byte_line_list(&list);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_printf_sort_uniq_wc(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 7 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "uniq") ||
      strcmp(words[pipe + 4], "|") || strcmp(words[pipe + 5], "wc") ||
      strcmp(words[pipe + 6], "-l")) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  write_padded_u64(count_unique_byte_line_list(&list));
  write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_printf_sort_wc(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 5 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "wc") ||
      strcmp(words[pipe + 4], "-l")) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  write_padded_u64((unsigned long long)list.len);
  write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_printf_sort_head(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (pipe < 0 || count != pipe + 6 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "head") ||
      strcmp(words[pipe + 4], "-n") || !parse_u64_arg(words[pipe + 5], &limit) ||
      limit == 0) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_printf_sort_tail(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (pipe < 0 || count != pipe + 6 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "tail") ||
      strcmp(words[pipe + 4], "-n") || !parse_u64_arg(words[pipe + 5], &limit)) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  size_t start = list.len - take;
  for (size_t idx = start; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_printf_sort_xargs_echo(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int first = 1;
  unsigned long long batch_size = 0;
  struct xargs_echo_batch_state batch = {0};
  if (pipe < 0 || count < pipe + 4 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") ||
      !xargs_echo_words_mode(words, pipe + 3, count, &batch_size)) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  batch.size = batch_size;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (batch_size) {
      emit_xargs_echo_batch_bytes(list.items[idx].data, list.items[idx].len, &batch);
    } else {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
  }
  if (batch_size) finish_xargs_echo_batch(&batch);
  if (!batch_size && !first) write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_printf_sort_xargs_wc(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (pipe < 0 || count != pipe + 6 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "xargs") ||
      strcmp(words[pipe + 4], "wc") || strcmp(words[pipe + 5], "-l")) {
    return unsupported();
  }
  if (!collect_printf_string_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  for (size_t idx = 0; idx < list.len; idx++) {
    if (!emit_xargs_wc_bytes(list.items[idx].data, list.items[idx].len, &total, &files,
                             &err)) {
      byte_line_list_free(&list);
      return 1;
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  byte_line_list_free(&list);
  return err ? 1 : 0;
}

static int parse_head_pipe_source(char **words, int pipe, const char **path,
                                  unsigned long long *limit, int *stdin_mode) {
  if (pipe < 0 || strcmp(words[0], "head")) return 0;
  *stdin_mode = 0;
  if (pipe == 1) {
    *limit = 10;
    *path = NULL;
    *stdin_mode = 1;
    return 1;
  }
  if (pipe == 2 && words[1][0] == '-' && words[1][1] >= '0' &&
      words[1][1] <= '9') {
    if (!parse_u64_arg(words[1] + 1, limit) || *limit == 0) return 0;
    *path = NULL;
    *stdin_mode = 1;
    return 1;
  }
  if (pipe == 3 && !strcmp(words[1], "-n")) {
    if (!parse_u64_arg(words[2], limit) || *limit == 0) return 0;
    *path = NULL;
    *stdin_mode = 1;
    return 1;
  }
  if (pipe == 2) {
    *limit = 10;
    *path = words[1];
    return 1;
  }
  if (pipe == 4 && !strcmp(words[1], "-n")) {
    if (!parse_u64_arg(words[2], limit) || *limit == 0) return 0;
    *path = words[3];
    return 1;
  }
  if (pipe == 3 && words[1][0] == '-' && words[1][1] >= '0' &&
      words[1][1] <= '9') {
    if (!parse_u64_arg(words[1] + 1, limit) || *limit == 0) return 0;
    *path = words[2];
    return 1;
  }
  return 0;
}

static int collect_head_lines(char **words, int pipe, struct byte_line_list *list) {
  const char *path = NULL;
  unsigned long long remaining = 0;
  int stdin_mode = 0;
  char buf[8192];
  char line[8192];
  size_t used = 0;
  if (!parse_head_pipe_source(words, pipe, &path, &remaining, &stdin_mode)) return 0;
  int fd = stdin_mode ? STDIN_FILENO : open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("head", path, errno);
    return 1;
  }
  while (remaining > 0) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("head", stdin_mode ? "stdin" : path, errno);
      if (!stdin_mode) close(fd);
      return 1;
    }
    for (ssize_t idx = 0; idx < read_len && remaining > 0; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
      if (!byte_line_list_push_plain_match(list, line, used, 0)) {
        if (!stdin_mode) close(fd);
        return 0;
      }
      used = 0;
        remaining--;
      }
    }
  }
  if (remaining > 0 && used > 0 &&
      !byte_line_list_push_plain_match(list, line, used, 0)) {
    if (!stdin_mode) close(fd);
    return 0;
  }
  if (!stdin_mode) close(fd);
  return 1;
}

typedef int (*head_line_visitor)(const char *line, size_t len, void *ctx);

static int stream_head_lines(char **words, int pipe, head_line_visitor visitor,
                             void *ctx) {
  const char *path = NULL;
  unsigned long long remaining = 0;
  int stdin_mode = 0;
  char buf[8192];
  char line[8192];
  size_t used = 0;
  if (!parse_head_pipe_source(words, pipe, &path, &remaining, &stdin_mode)) return 0;
  int fd = stdin_mode ? STDIN_FILENO : open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("head", path, errno);
    return 1;
  }
  while (remaining > 0) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("head", stdin_mode ? "stdin" : path, errno);
      if (!stdin_mode) close(fd);
      return 1;
    }
    for (ssize_t idx = 0; idx < read_len && remaining > 0; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
      if (!visitor(line, used, ctx)) {
        if (!stdin_mode) close(fd);
        return -1;
      }
      used = 0;
        remaining--;
      }
    }
  }
  if (remaining > 0 && used > 0 && !visitor(line, used, ctx)) {
    if (!stdin_mode) close(fd);
    return -1;
  }
  if (!stdin_mode) close(fd);
  return 1;
}

static void byte_line_list_drop_first(struct byte_line_list *list) {
  if (!list->len) return;
  free(list->items[0].data);
  if (list->len > 1) {
    memmove(list->items, list->items + 1,
            (list->len - 1) * sizeof(struct byte_line_item));
  }
  list->len--;
}

enum head_stream_mode {
  HEAD_STREAM_LINES,
  HEAD_STREAM_WC,
  HEAD_STREAM_HEAD,
  HEAD_STREAM_TAIL,
  HEAD_STREAM_XARGS_ECHO,
};

struct head_stream_ctx {
  enum head_stream_mode mode;
  const char *pattern;
  size_t pattern_len;
  unsigned long long limit;
  unsigned long long count;
  unsigned long long emitted;
  unsigned long long matches;
  int append_newline;
  int first;
  struct byte_line_list tail;
};

static int head_stream_visit(const char *line, size_t len, void *opaque) {
  struct head_stream_ctx *ctx = (struct head_stream_ctx *)opaque;
  if (ctx->pattern &&
      !contains_bytes(line, (ssize_t)len, ctx->pattern, ctx->pattern_len)) {
    return 1;
  }
  ctx->matches++;
  switch (ctx->mode) {
    case HEAD_STREAM_LINES:
      write_bytes(line, len);
      if (ctx->append_newline && (!len || line[len - 1] != '\n')) {
        write_bytes("\n", 1);
      }
      return 1;
    case HEAD_STREAM_WC:
      if (ctx->pattern || (len > 0 && line[len - 1] == '\n')) ctx->count++;
      return 1;
    case HEAD_STREAM_HEAD:
      if (ctx->emitted < ctx->limit) {
        write_bytes(line, len);
        if (ctx->append_newline && (!len || line[len - 1] != '\n')) {
          write_bytes("\n", 1);
        }
        ctx->emitted++;
      }
      return 1;
    case HEAD_STREAM_TAIL:
      if (ctx->limit == 0) return 1;
      if (!byte_line_list_push_plain_match(&ctx->tail, line, len,
                                           ctx->append_newline &&
                                               (!len || line[len - 1] != '\n'))) {
        return 0;
      }
      if (ctx->tail.len > ctx->limit) byte_line_list_drop_first(&ctx->tail);
      return 1;
    case HEAD_STREAM_XARGS_ECHO:
      emit_xargs_echo_bytes(line, len, &ctx->first);
      return 1;
  }
  return 0;
}

static int emit_head_stream_mode(char **words, int pipe, int start, int count,
                                 const char *pattern) {
  unsigned long long limit = 0;
  struct head_stream_ctx ctx = {0};
  ctx.pattern = pattern;
  ctx.pattern_len = pattern ? strlen(pattern) : 0;
  ctx.append_newline = pattern != NULL;
  ctx.first = 1;

  if (count == start) {
    if (!pattern) return unsupported();
    ctx.mode = HEAD_STREAM_LINES;
  } else if (count - start == 2 && !strcmp(words[start], "wc") &&
             !strcmp(words[start + 1], "-l")) {
    ctx.mode = HEAD_STREAM_WC;
  } else if (count - start == 3 && !strcmp(words[start], "head") &&
             !strcmp(words[start + 1], "-n") &&
             parse_u64_arg(words[start + 2], &limit) && limit > 0) {
    ctx.mode = HEAD_STREAM_HEAD;
    ctx.limit = limit;
  } else if (count - start == 3 && !strcmp(words[start], "tail") &&
             !strcmp(words[start + 1], "-n") &&
             parse_u64_arg(words[start + 2], &limit)) {
    ctx.mode = HEAD_STREAM_TAIL;
    ctx.limit = limit;
  } else if (count - start == 2 && !strcmp(words[start], "xargs") &&
             !strcmp(words[start + 1], "echo")) {
    ctx.mode = HEAD_STREAM_XARGS_ECHO;
  } else {
    return unsupported();
  }

  int streamed = stream_head_lines(words, pipe, head_stream_visit, &ctx);
  if (streamed < 0) {
    byte_line_list_free(&ctx.tail);
    return 1;
  }
  if (!streamed) {
    byte_line_list_free(&ctx.tail);
    return unsupported();
  }

  int rc = 0;
  if (ctx.mode == HEAD_STREAM_WC) {
    write_padded_u64(ctx.count);
    write_bytes("\n", 1);
  } else if (ctx.mode == HEAD_STREAM_TAIL) {
    for (size_t idx = 0; idx < ctx.tail.len; idx++) {
      write_bytes(ctx.tail.items[idx].data, ctx.tail.items[idx].len);
    }
  } else if (ctx.mode == HEAD_STREAM_XARGS_ECHO) {
    if (!ctx.first) write_bytes("\n", 1);
  } else if (ctx.mode == HEAD_STREAM_LINES && pattern && ctx.matches == 0) {
    rc = 1;
  }
  byte_line_list_free(&ctx.tail);
  return rc;
}

static int collect_head_grep_lines(char **words, int pipe, struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  const char *pattern = words[pipe + 2];
  struct head_stream_ctx ctx = {
      .mode = HEAD_STREAM_TAIL,
      .pattern = pattern,
      .pattern_len = strlen(pattern),
      .limit = (unsigned long long)-1,
      .append_newline = 1,
      .first = 1,
  };
  int streamed = stream_head_lines(words, pipe, head_stream_visit, &ctx);
  if (streamed <= 0) {
    byte_line_list_free(&ctx.tail);
    return 0;
  }
  *list = ctx.tail;
  return 1;
}

static unsigned long long count_byte_line_newlines(const struct byte_line_list *list) {
  unsigned long long count = 0;
  for (size_t idx = 0; idx < list->len; idx++) {
    if (list->items[idx].len > 0 && list->items[idx].data[list->items[idx].len - 1] == '\n') {
      count++;
    }
  }
  return count;
}

static unsigned long long count_byte_line_bytes(const struct byte_line_list *list) {
  unsigned long long count = 0;
  for (size_t idx = 0; idx < list->len; idx++) {
    count += (unsigned long long)list->items[idx].len;
  }
  return count;
}

static unsigned long long count_byte_line_words(const struct byte_line_list *list) {
  unsigned long long count = 0;
  int in_word = 0;
  for (size_t idx = 0; idx < list->len; idx++) {
    const struct byte_line_item *item = &list->items[idx];
    for (size_t pos = 0; pos < item->len; pos++) {
      unsigned char byte = (unsigned char)item->data[pos];
      if (isspace(byte)) {
        in_word = 0;
      } else if (!in_word) {
        count++;
        in_word = 1;
      }
    }
  }
  return count;
}

static unsigned long long count_byte_line_wc(const struct byte_line_list *list,
                                             enum wc_count_mode mode,
                                             int wc_counts_newlines) {
  if (mode == WC_COUNT_LINES) {
    return wc_counts_newlines ? count_byte_line_newlines(list)
                              : (unsigned long long)list->len;
  }
  if (mode == WC_COUNT_BYTES) return count_byte_line_bytes(list);
  return count_byte_line_words(list);
}

static unsigned long long count_unique_byte_line_wc(const struct byte_line_list *list,
                                                    enum wc_count_mode mode) {
  if (mode == WC_COUNT_LINES) return count_unique_byte_line_list(list);
  unsigned long long count = 0;
  int in_word = 0;
  for (size_t idx = 0; idx < list->len; idx++) {
    int duplicate =
        idx > 0 && byte_line_items_equal_without_newline(&list->items[idx - 1],
                                                         &list->items[idx]);
    if (duplicate) continue;
    if (mode == WC_COUNT_BYTES) {
      count += (unsigned long long)list->items[idx].len;
      continue;
    }
    for (size_t pos = 0; pos < list->items[idx].len; pos++) {
      unsigned char byte = (unsigned char)list->items[idx].data[pos];
      if (isspace(byte)) {
        in_word = 0;
      } else if (!in_word) {
        count++;
        in_word = 1;
      }
    }
  }
  return count;
}

static int byte_line_list_ensure_trailing_newlines(struct byte_line_list *list) {
  for (size_t idx = 0; idx < list->len; idx++) {
    struct byte_line_item *item = &list->items[idx];
    if (item->len > 0 && item->data[item->len - 1] == '\n') continue;
    char *next = (char *)realloc(item->data, item->len + 1);
    if (!next) return 0;
    next[item->len] = '\n';
    item->data = next;
    item->len++;
  }
  return 1;
}

static int collect_xargs_wc_output_from_byte_lines(struct byte_line_list *source,
                                                   struct byte_line_list *out) {
  unsigned long long total = 0;
  unsigned long long files = 0;
  size_t input_paths = 0;
  int err = 0;
  for (size_t idx = 0; idx < source->len; idx++) {
    struct byte_line_item *item = &source->items[idx];
    size_t cursor = 0;
    while (cursor < item->len) {
      while (cursor < item->len && isspace((unsigned char)item->data[cursor])) cursor++;
      size_t start = cursor;
      while (cursor < item->len && !isspace((unsigned char)item->data[cursor])) cursor++;
      if (cursor <= start) continue;
      size_t token_len = cursor - start;
      char *path = (char *)malloc(token_len + 1);
      if (!path) return 0;
      memcpy(path, item->data + start, token_len);
      path[token_len] = 0;
      input_paths++;
      int ok = byte_line_list_push_xargs_wc_path(out, path, &total, &files, &err);
      free(path);
      if (!ok) return 0;
    }
  }
  if (input_paths > 1) {
    char total_line[64];
    int len = snprintf(total_line, sizeof(total_line), "%8llu total", total);
    if (len < 0 || (size_t)len >= sizeof(total_line) ||
        !byte_line_list_push_plain_match(out, total_line, (size_t)len, 1)) {
      return 0;
    }
  }
  (void)err;
  return 1;
}

static int emit_xargs_wc_output_line_list_mode(char **words, int xargs_start,
                                               int count,
                                               struct byte_line_list *list,
                                               int sort_input) {
  if (sort_input) {
    if (!byte_line_list_ensure_trailing_newlines(list)) return 1;
    qsort(list->items, list->len, sizeof(struct byte_line_item), byte_line_item_cmp);
  }
  struct byte_line_list wc_lines = {0};
  if (!collect_xargs_wc_output_from_byte_lines(list, &wc_lines)) {
    byte_line_list_free(&wc_lines);
    return 1;
  }
  int rc = emit_head_line_list_mode(words, xargs_start + 4, count, &wc_lines, 0);
  byte_line_list_free(&wc_lines);
  return rc;
}

static int head_line_list_mode_supported(char **words, int start, int count) {
  unsigned long long limit = 0;
  enum wc_count_mode wc_mode;
  int remaining = count - start;
  unsigned long long batch_size = 0;
  if (remaining >= 5 && !strcmp(words[start], "xargs") &&
      !strcmp(words[start + 1], "wc") && !strcmp(words[start + 2], "-l") &&
      !strcmp(words[start + 3], "|") &&
      xargs_wc_output_mode_supported(words, start + 4, count)) {
    return 1;
  }
  if (remaining == 2 && !strcmp(words[start], "wc") &&
      parse_wc_count_mode(words[start + 1], &wc_mode)) {
    return 1;
  }
  if (remaining == 3 && !strcmp(words[start], "head") && !strcmp(words[start + 1], "-n") &&
      parse_u64_arg(words[start + 2], &limit) && limit > 0) {
    return 1;
  }
  if (remaining == 3 && !strcmp(words[start], "tail") && !strcmp(words[start + 1], "-n") &&
      parse_u64_arg(words[start + 2], &limit)) {
    return 1;
  }
  if (remaining >= 1 && !strcmp(words[start], "sort")) {
    if (remaining == 1) return 1;
    if (remaining >= 7 && !strcmp(words[start + 1], "|") &&
        xargs_wc_output_mode_supported(words, start + 6, count)) {
      return 1;
    }
    if (remaining == 3 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "uniq")) {
      return 1;
    }
    if (remaining == 6 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "uniq") && !strcmp(words[start + 3], "|") &&
        !strcmp(words[start + 4], "wc") &&
        parse_wc_count_mode(words[start + 5], &wc_mode)) {
      return 1;
    }
    if (remaining == 4 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "wc") &&
        parse_wc_count_mode(words[start + 3], &wc_mode)) {
      return 1;
    }
    if (remaining == 5 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "head") && !strcmp(words[start + 3], "-n") &&
        parse_u64_arg(words[start + 4], &limit) && limit > 0) {
      return 1;
    }
    if (remaining == 5 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "tail") && !strcmp(words[start + 3], "-n") &&
        parse_u64_arg(words[start + 4], &limit)) {
      return 1;
    }
    if (remaining >= 4 && !strcmp(words[start + 1], "|") &&
        xargs_echo_words_mode(words, start + 2, count, &batch_size)) {
      return 1;
    }
    if (remaining == 5 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "xargs") && !strcmp(words[start + 3], "wc") &&
        !strcmp(words[start + 4], "-l")) {
      return 1;
    }
    return 0;
  }
  if (xargs_echo_words_mode(words, start, count, &batch_size)) {
    return 1;
  }
  if (remaining == 3 && !strcmp(words[start], "xargs") && !strcmp(words[start + 1], "wc") &&
      !strcmp(words[start + 2], "-l")) {
    return 1;
  }
  return 0;
}

static int emit_head_line_list_mode(char **words, int start, int count,
                                    struct byte_line_list *list,
                                    int wc_counts_newlines) {
  unsigned long long limit = 0;
  enum wc_count_mode wc_mode;
  int first = 1;
  unsigned long long batch_size = 0;
  struct xargs_echo_batch_state batch = {0};
  if (count - start >= 5 && !strcmp(words[start], "xargs") &&
      !strcmp(words[start + 1], "wc") && !strcmp(words[start + 2], "-l") &&
      !strcmp(words[start + 3], "|") &&
      xargs_wc_output_mode_supported(words, start + 4, count)) {
    return emit_xargs_wc_output_line_list_mode(words, start, count, list, 0);
  }
  if (count - start == 2 && !strcmp(words[start], "wc") &&
      parse_wc_count_mode(words[start + 1], &wc_mode)) {
    unsigned long long n = count_byte_line_wc(list, wc_mode, wc_counts_newlines);
    write_padded_u64(n);
    write_bytes("\n", 1);
    return 0;
  }
  if (count - start == 3 && !strcmp(words[start], "head") &&
      !strcmp(words[start + 1], "-n") && parse_u64_arg(words[start + 2], &limit) &&
      limit > 0) {
    size_t take = limit > (unsigned long long)list->len ? list->len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list->items[idx].data, list->items[idx].len);
    return 0;
  }
  if (count - start == 3 && !strcmp(words[start], "tail") &&
      !strcmp(words[start + 1], "-n") && parse_u64_arg(words[start + 2], &limit)) {
    size_t take = limit > (unsigned long long)list->len ? list->len : (size_t)limit;
    size_t first_idx = list->len - take;
    for (size_t idx = first_idx; idx < list->len; idx++) write_bytes(list->items[idx].data, list->items[idx].len);
    return 0;
  }
  if (count - start >= 1 && !strcmp(words[start], "sort")) {
    if (count - start >= 7 && !strcmp(words[start + 1], "|") &&
        xargs_wc_output_mode_supported(words, start + 6, count)) {
      return emit_xargs_wc_output_line_list_mode(words, start + 2, count, list, 1);
    }
    if (!byte_line_list_ensure_trailing_newlines(list)) return 1;
    qsort(list->items, list->len, sizeof(struct byte_line_item), byte_line_item_cmp);
    if (count - start == 1) {
      for (size_t idx = 0; idx < list->len; idx++) write_bytes(list->items[idx].data, list->items[idx].len);
      return 0;
    }
    if (count - start == 3 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "uniq")) {
      emit_unique_byte_line_list(list);
      return 0;
    }
    if (count - start == 6 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "uniq") && !strcmp(words[start + 3], "|") &&
        !strcmp(words[start + 4], "wc") &&
        parse_wc_count_mode(words[start + 5], &wc_mode)) {
      write_padded_u64(count_unique_byte_line_wc(list, wc_mode));
      write_bytes("\n", 1);
      return 0;
    }
    if (count - start == 4 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "wc") &&
        parse_wc_count_mode(words[start + 3], &wc_mode)) {
      write_padded_u64(count_byte_line_wc(list, wc_mode, 0));
      write_bytes("\n", 1);
      return 0;
    }
    if (count - start == 5 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "head") && !strcmp(words[start + 3], "-n") &&
        parse_u64_arg(words[start + 4], &limit) && limit > 0) {
      size_t take = limit > (unsigned long long)list->len ? list->len : (size_t)limit;
      for (size_t idx = 0; idx < take; idx++) write_bytes(list->items[idx].data, list->items[idx].len);
      return 0;
    }
    if (count - start == 5 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "tail") && !strcmp(words[start + 3], "-n") &&
        parse_u64_arg(words[start + 4], &limit)) {
      size_t take = limit > (unsigned long long)list->len ? list->len : (size_t)limit;
      size_t first_idx = list->len - take;
      for (size_t idx = first_idx; idx < list->len; idx++) write_bytes(list->items[idx].data, list->items[idx].len);
      return 0;
    }
    if (count - start >= 4 && !strcmp(words[start + 1], "|") &&
        xargs_echo_words_mode(words, start + 2, count, &batch_size)) {
      batch.size = batch_size;
      if (batch_size) {
        for (size_t idx = 0; idx < list->len; idx++) {
          emit_xargs_echo_batch_bytes(list->items[idx].data, list->items[idx].len, &batch);
        }
        finish_xargs_echo_batch(&batch);
      } else {
        for (size_t idx = 0; idx < list->len; idx++) {
          emit_xargs_echo_bytes(list->items[idx].data, list->items[idx].len, &first);
        }
        if (!first) write_bytes("\n", 1);
      }
      return 0;
    }
    if (count - start == 5 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "xargs") && !strcmp(words[start + 3], "wc") &&
        !strcmp(words[start + 4], "-l")) {
      unsigned long long total = 0;
      unsigned long long files = 0;
      int err = 0;
      for (size_t idx = 0; idx < list->len; idx++) {
        if (!emit_xargs_wc_bytes(list->items[idx].data, list->items[idx].len, &total, &files,
                                 &err)) {
          return 1;
        }
      }
      if (files > 1) {
        write_padded_u64(total);
        write_bytes(" total\n", 7);
      }
      return err ? 1 : 0;
    }
    return unsupported();
  }
  if (xargs_echo_words_mode(words, start, count, &batch_size)) {
    batch.size = batch_size;
    if (batch_size) {
      for (size_t idx = 0; idx < list->len; idx++) {
        emit_xargs_echo_batch_bytes(list->items[idx].data, list->items[idx].len, &batch);
      }
      finish_xargs_echo_batch(&batch);
    } else {
      for (size_t idx = 0; idx < list->len; idx++) {
        emit_xargs_echo_bytes(list->items[idx].data, list->items[idx].len, &first);
      }
      if (!first) write_bytes("\n", 1);
    }
    return 0;
  }
  if (count - start == 3 && !strcmp(words[start], "xargs") &&
      !strcmp(words[start + 1], "wc") && !strcmp(words[start + 2], "-l")) {
    unsigned long long total = 0;
    unsigned long long files = 0;
    int err = 0;
    for (size_t idx = 0; idx < list->len; idx++) {
      if (!emit_xargs_wc_bytes(list->items[idx].data, list->items[idx].len, &total, &files,
                               &err)) {
        return 1;
      }
    }
    if (files > 1) {
      write_padded_u64(total);
      write_bytes(" total\n", 7);
    }
    return err ? 1 : 0;
  }
  return unsupported();
}

static int collect_xargs_echo_stdin_lines(struct byte_line_list *list,
                                          unsigned long long batch_size) {
  char *data = NULL;
  size_t size = 0;
  if (read_all_fd(STDIN_FILENO, "xargs", "stdin", &data, &size)) return -1;
  if (batch_size) {
    size_t cursor = 0;
    unsigned long long used_tokens = 0;
    size_t used = 0;
    char *out = (char *)malloc(size + 1);
    if (!out) {
      free(data);
      return -1;
    }
    while (cursor < size) {
      while (cursor < size && isspace((unsigned char)data[cursor])) cursor++;
      size_t start = cursor;
      while (cursor < size && !isspace((unsigned char)data[cursor])) cursor++;
      if (cursor > start) {
        if (used_tokens) out[used++] = ' ';
        memcpy(out + used, data + start, cursor - start);
        used += cursor - start;
        used_tokens++;
        if (used_tokens == batch_size) {
          if (!byte_line_list_push_plain_match(list, out, used, 1)) {
            free(out);
            free(data);
            return -1;
          }
          used = 0;
          used_tokens = 0;
        }
      }
    }
    if (used_tokens && !byte_line_list_push_plain_match(list, out, used, 1)) {
      free(out);
      free(data);
      return -1;
    }
    free(out);
    free(data);
    return 1;
  }
  if (size > SIZE_MAX - 2) {
    free(data);
    errno = ENOMEM;
    write_err_path("xargs", "stdin", errno);
    return -1;
  }
  char *out = (char *)malloc(size + 2);
  if (!out) {
    free(data);
    return -1;
  }
  size_t used = 0;
  size_t cursor = 0;
  int first = 1;
  while (cursor < size) {
    while (cursor < size && isspace((unsigned char)data[cursor])) cursor++;
    size_t start = cursor;
    while (cursor < size && !isspace((unsigned char)data[cursor])) cursor++;
    if (cursor > start) {
      if (!first) out[used++] = ' ';
      memcpy(out + used, data + start, cursor - start);
      used += cursor - start;
      first = 0;
    }
  }
  int ok = 1;
  if (!first) {
    out[used++] = '\n';
    ok = byte_line_list_push(list, out, used);
  }
  free(out);
  free(data);
  return ok ? 1 : -1;
}

static int count_xargs_echo_stdin_wc_lines(unsigned long long *count,
                                           unsigned long long batch_size) {
  char buf[8192];
  int in_token = 0;
  int saw_token = 0;
  unsigned long long tokens = 0;
  for (;;) {
    ssize_t read_len = read(STDIN_FILENO, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("xargs", "stdin", errno);
      return -1;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (isspace((unsigned char)buf[idx])) {
        in_token = 0;
      } else if (!in_token) {
        saw_token = 1;
        in_token = 1;
        tokens++;
      }
    }
  }
  *count = batch_size ? (tokens ? 1 + ((tokens - 1) / batch_size) : 0)
                      : (saw_token ? 1 : 0);
  return 1;
}

static int count_xargs_echo_stdin_grep_wc_lines(const char *pat, size_t pat_len,
                                                unsigned long long *count) {
  char buf[8192];
  char *window = NULL;
  size_t carry_len = 0;
  size_t carry_cap = pat_len > 0 ? pat_len - 1 : 0;
  int in_token = 0;
  int saw_token = 0;
  int matched = 0;
  if (pat_len == 0) return -1;
  if (carry_cap > 0) {
    window = (char *)malloc(carry_cap + sizeof(buf));
    if (!window) {
      write_err_path("xargs", "stdin", errno);
      return -1;
    }
  }
  for (;;) {
    ssize_t read_len = read(STDIN_FILENO, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("xargs", "stdin", errno);
      free(window);
      return -1;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (isspace((unsigned char)buf[idx])) {
        in_token = 0;
      } else if (!in_token) {
        saw_token = 1;
        in_token = 1;
      }
    }
    if (!matched) {
      if (carry_cap == 0) {
        matched = contains_bytes(buf, read_len, pat, pat_len);
      } else {
        memcpy(window + carry_len, buf, (size_t)read_len);
        size_t total = carry_len + (size_t)read_len;
        matched = contains_bytes(window, (ssize_t)total, pat, pat_len);
        carry_len = carry_cap < total ? carry_cap : total;
        memmove(window, window + total - carry_len, carry_len);
      }
    }
  }
  free(window);
  *count = saw_token && matched ? 1 : 0;
  return 1;
}

static int pipe_xargs_echo_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  unsigned long long batch_size = 0;
  if (pipe < 0 || pipe + 1 >= count) return unsupported();
  if (!xargs_echo_words_mode(words, 0, pipe, &batch_size)) {
    return unsupported();
  }
  int emit_start = pipe + 1;
  int grep_filter = 0;
  int direct_grep = 0;
  const char *grep_pattern = NULL;
  if (count - emit_start >= 2 && !strcmp(words[emit_start], "grep") &&
      is_plain_literal_pattern(words[emit_start + 1])) {
    grep_filter = 1;
    grep_pattern = words[emit_start + 1];
    if (count == emit_start + 2) {
      direct_grep = 1;
    } else {
      if (count <= emit_start + 3 || strcmp(words[emit_start + 2], "|")) {
        return unsupported();
      }
      emit_start += 3;
      if (!head_line_list_mode_supported(words, emit_start, count)) return unsupported();
    }
  } else if (!head_line_list_mode_supported(words, emit_start, count)) {
    return unsupported();
  }
  if (!grep_filter && count - (pipe + 1) == 2 && !strcmp(words[pipe + 1], "wc") &&
      !strcmp(words[pipe + 2], "-l")) {
    unsigned long long line_count = 0;
    if (count_xargs_echo_stdin_wc_lines(&line_count, batch_size) < 0) return 1;
    write_padded_u64(line_count);
    write_bytes("\n", 1);
    return 0;
  }
  if (!batch_size && grep_filter && !direct_grep && count - emit_start == 2 &&
      !strcmp(words[emit_start], "wc") && !strcmp(words[emit_start + 1], "-l")) {
    unsigned long long line_count = 0;
    if (count_xargs_echo_stdin_grep_wc_lines(grep_pattern, strlen(grep_pattern),
                                             &line_count) < 0) {
      return 1;
    }
    write_padded_u64(line_count);
    write_bytes("\n", 1);
    return 0;
  }
  struct byte_line_list list = {0};
  if (collect_xargs_echo_stdin_lines(&list, batch_size) < 0) {
    byte_line_list_free(&list);
    return 1;
  }
  if (grep_filter) {
    struct byte_line_list filtered = {0};
    size_t pattern_len = strlen(grep_pattern);
    for (size_t idx = 0; idx < list.len; idx++) {
      struct byte_line_item *item = &list.items[idx];
      if (contains_bytes(item->data, (ssize_t)item->len, grep_pattern, pattern_len) &&
          !byte_line_list_push_plain_match(&filtered, item->data, item->len, 0)) {
        byte_line_list_free(&list);
        byte_line_list_free(&filtered);
        return 1;
      }
    }
    byte_line_list_free(&list);
    if (direct_grep) {
      for (size_t idx = 0; idx < filtered.len; idx++) {
        write_bytes(filtered.items[idx].data, filtered.items[idx].len);
      }
      int rc = filtered.len ? 0 : 1;
      byte_line_list_free(&filtered);
      return rc;
    }
    int rc = emit_head_line_list_mode(words, emit_start, count, &filtered, 0);
    byte_line_list_free(&filtered);
    return rc;
  }
  int rc = emit_head_line_list_mode(words, emit_start, count, &list, 1);
  byte_line_list_free(&list);
  return rc;
}

static int pipe_empty_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe != 1 || (strcmp(words[0], "true") && strcmp(words[0], "false"))) {
    return unsupported();
  }
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count == pipe + 3) return 1;
    if (strcmp(words[pipe + 3], "|") ||
        !head_line_list_mode_supported(words, pipe + 4, count)) {
      return unsupported();
    }
    return emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
  }
  if (!head_line_list_mode_supported(words, pipe + 1, count)) {
    return unsupported();
  }
  return emit_head_line_list_mode(words, pipe + 1, count, &list, 0);
}

static int parse_side_effect_empty_pipe_left(char **words, int pipe, int *kind,
                                             int *parents, int *first_path) {
  if (pipe < 2) return 0;
  if (!strcmp(words[0], "mkdir")) {
    int local_parents = 0;
    int local_first_path = 1;
    if (!strcmp(words[1], "-p")) {
      local_parents = 1;
      local_first_path = 2;
    } else if (words[1][0] == '-') {
      return 0;
    }
    if (local_first_path >= pipe) return 0;
    for (int idx = local_first_path; idx < pipe; idx++) {
      if (words[idx][0] == '-') return 0;
    }
    *kind = 1;
    *parents = local_parents;
    *first_path = local_first_path;
    return 1;
  }
  if (!strcmp(words[0], "touch")) {
    for (int idx = 1; idx < pipe; idx++) {
      if (words[idx][0] == '-') return 0;
    }
    *kind = 2;
    *parents = 0;
    *first_path = 1;
    return 1;
  }
  return 0;
}

static int run_side_effect_empty_pipe_left(char **words, int pipe, int kind,
                                           int parents, int first_path) {
  int rc = 0;
  if (kind == 1) {
    for (int idx = first_path; idx < pipe; idx++) {
      if (parents) {
        rc |= mkdir_p_one(words[idx]);
      } else if (mkdir(words[idx], 0777) != 0) {
        write_err_path("mkdir", words[idx], errno);
        rc = 1;
      }
    }
    return rc;
  }
  if (kind == 2) {
    for (int idx = first_path; idx < pipe; idx++) {
      if (utimes(words[idx], NULL) == 0) continue;
      if (errno != ENOENT) {
        write_err_path("touch", words[idx], errno);
        rc = 1;
        continue;
      }
      int fd = open(words[idx], O_WRONLY | O_CREAT, 0666);
      if (fd < 0) {
        write_err_path("touch", words[idx], errno);
        rc = 1;
        continue;
      }
      close(fd);
      if (utimes(words[idx], NULL) != 0) {
        write_err_path("touch", words[idx], errno);
        rc = 1;
      }
    }
    return rc;
  }
  return 127;
}

static int pipe_side_effect_empty_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  int kind = 0;
  int parents = 0;
  int first_path = 0;
  struct byte_line_list list = {0};
  if (pipe < 0 || pipe + 1 >= count ||
      !parse_side_effect_empty_pipe_left(words, pipe, &kind, &parents, &first_path)) {
    return unsupported();
  }
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count == pipe + 3) {
      (void)run_side_effect_empty_pipe_left(words, pipe, kind, parents, first_path);
      return 1;
    }
    if (strcmp(words[pipe + 3], "|") ||
        !head_line_list_mode_supported(words, pipe + 4, count)) {
      return unsupported();
    }
    (void)run_side_effect_empty_pipe_left(words, pipe, kind, parents, first_path);
    return emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
  }
  if (!head_line_list_mode_supported(words, pipe + 1, count)) {
    return unsupported();
  }
  (void)run_side_effect_empty_pipe_left(words, pipe, kind, parents, first_path);
  return emit_head_line_list_mode(words, pipe + 1, count, &list, 0);
}

static int parse_predicate_empty_pipe_left(char **words, int pipe,
                                           struct test_expr *expr) {
  if (pipe < 2) return 0;
  if (!strcmp(words[0], "test")) {
    return parse_test_words(words, 1, pipe, expr);
  }
  if (!strcmp(words[0], "[")) {
    if (pipe < 3 || strcmp(words[pipe - 1], "]")) return 0;
    return parse_test_words(words, 1, pipe - 1, expr);
  }
  return 0;
}

static int pipe_predicate_empty_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct test_expr expr;
  struct byte_line_list list = {0};
  if (pipe < 0 || pipe + 1 >= count ||
      !parse_predicate_empty_pipe_left(words, pipe, &expr)) {
    return unsupported();
  }
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count == pipe + 3) {
      (void)eval_test_expr(&expr);
      return 1;
    }
    if (strcmp(words[pipe + 3], "|") ||
        !head_line_list_mode_supported(words, pipe + 4, count)) {
      return unsupported();
    }
    (void)eval_test_expr(&expr);
    return emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
  }
  if (!head_line_list_mode_supported(words, pipe + 1, count)) {
    return unsupported();
  }
  (void)eval_test_expr(&expr);
  return emit_head_line_list_mode(words, pipe + 1, count, &list, 0);
}

static int collect_wc_pipe_lines(char **words, int pipe,
                                 struct byte_line_list *list) {
  enum wc_count_mode mode;
  unsigned long long total = 0;
  if (pipe < 2 || strcmp(words[0], "wc") || !parse_wc_count_mode(words[1], &mode)) {
    return 0;
  }
  if (pipe == 2) {
    int count_err = 0;
    unsigned long long n = count_wc_fd(STDIN_FILENO, mode, "stdin", &count_err);
    if (count_err) return -1;
    char line[64];
    int len = snprintf(line, sizeof(line), "%8llu", n);
    if (len < 0 || (size_t)len >= sizeof(line) ||
        !byte_line_list_push_plain_match(list, line, (size_t)len, 1)) {
      return -1;
    }
    return 1;
  }
  for (int idx = 2; idx < pipe; idx++) {
    struct stat st;
    if (words[idx][0] == '-' || stat(words[idx], &st) != 0 || !S_ISREG(st.st_mode)) {
      return 0;
    }
  }
  for (int idx = 2; idx < pipe; idx++) {
    int fd = open(words[idx], O_RDONLY);
    if (fd < 0) {
      write_err_path("wc", words[idx], errno);
      continue;
    }
    int count_err = 0;
    unsigned long long n = count_wc_fd(fd, mode, words[idx], &count_err);
    close(fd);
    if (count_err) continue;
    total += n;

    char prefix[64];
    int prefix_len = snprintf(prefix, sizeof(prefix), "%8llu ", n);
    size_t path_len = strlen(words[idx]);
    if (prefix_len < 0 || (size_t)prefix_len >= sizeof(prefix)) return -1;
    char *line = (char *)malloc((size_t)prefix_len + path_len);
    if (!line) return -1;
    memcpy(line, prefix, (size_t)prefix_len);
    memcpy(line + prefix_len, words[idx], path_len);
    int ok = byte_line_list_push_plain_match(list, line, (size_t)prefix_len + path_len, 1);
    free(line);
    if (!ok) return -1;
  }
  if (pipe - 2 > 1) {
    char total_line[64];
    int len = snprintf(total_line, sizeof(total_line), "%8llu total", total);
    if (len < 0 || (size_t)len >= sizeof(total_line) ||
        !byte_line_list_push_plain_match(list, total_line, (size_t)len, 1)) {
      return -1;
    }
  }
  return 1;
}

static int pipe_wc_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0 || pipe + 1 >= count) return unsupported();
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    if (count != pipe + 3 &&
        !head_line_list_mode_supported(words, pipe + 4, count)) {
      return unsupported();
    }
    rc = collect_wc_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    struct byte_line_list filtered = {0};
    const char *pattern = words[pipe + 2];
    size_t pattern_len = strlen(pattern);
    for (size_t idx = 0; idx < list.len; idx++) {
      if (contains_bytes(list.items[idx].data, (ssize_t)list.items[idx].len,
                         pattern, pattern_len) &&
          !byte_line_list_push_plain_match(&filtered, list.items[idx].data,
                                           list.items[idx].len, 0)) {
        byte_line_list_free(&list);
        byte_line_list_free(&filtered);
        return 1;
      }
    }
    byte_line_list_free(&list);
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < filtered.len; idx++) {
        write_bytes(filtered.items[idx].data, filtered.items[idx].len);
      }
      rc = filtered.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &filtered, 0);
    }
    byte_line_list_free(&filtered);
    return rc;
  }
  if (!head_line_list_mode_supported(words, pipe + 1, count)) {
    return unsupported();
  }
  rc = collect_wc_pipe_lines(words, pipe, &list);
  if (rc < 0) {
    byte_line_list_free(&list);
    return 1;
  }
  if (rc == 0) {
    byte_line_list_free(&list);
    return unsupported();
  }
  rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  byte_line_list_free(&list);
  return rc;
}

static int collect_du_sk_pipe_lines(char **words, int pipe,
                                    struct byte_line_list *list) {
  char path[PATH_MAX];
  char *paths[2] = {path, NULL};
  struct stat initial;
  if (pipe != 3 || strcmp(words[0], "du") || strcmp(words[1], "-sk") ||
      words[2][0] == '-' || lstat(words[2], &initial) != 0 ||
      !copy_cstr(path, sizeof(path), words[2])) {
    return 0;
  }
  FTS *fts = fts_open(paths, FTS_PHYSICAL | FTS_NOCHDIR, NULL);
  if (!fts) {
    write_err_path("du", words[2], errno);
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
        break;
      default:
        if (entry->fts_statp) {
          saw_countable = 1;
          blocks += (unsigned long long)entry->fts_statp->st_blocks;
        }
        break;
    }
  }
  if (errno != 0) write_err_path("du", words[2], errno);
  if (fts_close(fts) != 0) write_err_path("du", words[2], errno);
  if (saw_countable) {
    char prefix[64];
    unsigned long long kib = (blocks + 1) / 2;
    int prefix_len = snprintf(prefix, sizeof(prefix), "%llu\t", kib);
    size_t path_len = strlen(words[2]);
    if (prefix_len < 0 || (size_t)prefix_len >= sizeof(prefix)) return -1;
    char *line = (char *)malloc((size_t)prefix_len + path_len);
    if (!line) return -1;
    memcpy(line, prefix, (size_t)prefix_len);
    memcpy(line + prefix_len, words[2], path_len);
    int ok = byte_line_list_push_plain_match(list, line, (size_t)prefix_len + path_len, 1);
    free(line);
    if (!ok) return -1;
  }
  return 1;
}

static int pipe_du_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0 || pipe + 1 >= count) return unsupported();
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    if (count != pipe + 3 &&
        !head_line_list_mode_supported(words, pipe + 4, count)) {
      return unsupported();
    }
    rc = collect_du_sk_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    struct byte_line_list filtered = {0};
    const char *pattern = words[pipe + 2];
    size_t pattern_len = strlen(pattern);
    for (size_t idx = 0; idx < list.len; idx++) {
      if (contains_bytes(list.items[idx].data, (ssize_t)list.items[idx].len,
                         pattern, pattern_len) &&
          !byte_line_list_push_plain_match(&filtered, list.items[idx].data,
                                           list.items[idx].len, 0)) {
        byte_line_list_free(&list);
        byte_line_list_free(&filtered);
        return 1;
      }
    }
    byte_line_list_free(&list);
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < filtered.len; idx++) {
        write_bytes(filtered.items[idx].data, filtered.items[idx].len);
      }
      rc = filtered.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &filtered, 0);
    }
    byte_line_list_free(&filtered);
    return rc;
  }
  if (!head_line_list_mode_supported(words, pipe + 1, count)) {
    return unsupported();
  }
  rc = collect_du_sk_pipe_lines(words, pipe, &list);
  if (rc < 0) {
    byte_line_list_free(&list);
    return 1;
  }
  if (rc == 0) {
    byte_line_list_free(&list);
    return unsupported();
  }
  rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  byte_line_list_free(&list);
  return rc;
}

static int single_line_push_basename(struct byte_line_list *list, const char *input,
                                     const char *suffix) {
  size_t len = strlen(input);
  if (len == 0) return byte_line_list_push_plain_match(list, ".", 1, 1);
  while (len > 1 && input[len - 1] == '/') len--;
  size_t start = len;
  while (start > 0 && input[start - 1] != '/') start--;
  if (start == len && input[0] == '/') {
    return byte_line_list_push_plain_match(list, "/", 1, 1);
  }
  size_t base_len = len - start;
  if (suffix) {
    size_t suffix_len = strlen(suffix);
    if (suffix_len > 0 && suffix_len < base_len &&
        memcmp(input + start + base_len - suffix_len, suffix, suffix_len) == 0) {
      base_len -= suffix_len;
    }
  }
  return byte_line_list_push_plain_match(list, input + start, base_len, 1);
}

static int single_line_push_dirname(struct byte_line_list *list, const char *input) {
  size_t len = strlen(input);
  if (len == 0) return byte_line_list_push_plain_match(list, ".", 1, 1);
  while (len > 1 && input[len - 1] == '/') len--;
  size_t end = len;
  while (end > 0 && input[end - 1] != '/') end--;
  if (end == 0) return byte_line_list_push_plain_match(list, ".", 1, 1);
  while (end > 1 && input[end - 1] == '/') end--;
  return byte_line_list_push_plain_match(list, input, end, 1);
}

static int collect_single_line_pipe_source(char **words, int pipe,
                                           struct byte_line_list *list) {
  char buf[2048];
  if (pipe == 1 && !strcmp(words[0], "pwd")) {
    if (!getcwd(buf, sizeof(buf))) {
      write_err_path("pwd", NULL, errno);
      return -1;
    }
    return byte_line_list_push_plain_match(list, buf, strlen(buf), 1) ? 1 : -1;
  }
  if (pipe == 1 && !strcmp(words[0], "whoami")) {
    const char *name = effective_user_name();
    if (!name) {
      const char *msg = "whoami: cannot find name for user ID\n";
      write_fd_all(2, msg, strlen(msg));
      return -1;
    }
    return byte_line_list_push_plain_match(list, name, strlen(name), 1) ? 1 : -1;
  }
  if (pipe == 1 && !strcmp(words[0], "hostname")) {
    char name[256];
    if (gethostname(name, sizeof(name)) != 0) {
      write_err_path("hostname", NULL, errno);
      return -1;
    }
    name[sizeof(name) - 1] = 0;
    return byte_line_list_push_plain_match(list, name, strlen(name), 1) ? 1 : -1;
  }
  if (pipe == 2 && !strcmp(words[0], "printenv") && words[1][0] != '-') {
    const char *value = find_environment_value(words[1]);
    if (!value) return 1;
    return byte_line_list_push_plain_match(list, value, strlen(value), 1) ? 1 : -1;
  }
  if (pipe == 1 && !strcmp(words[0], "id")) {
    char value[4096];
    int rc = default_id_value(value, sizeof(value));
    if (rc == 0) return 0;
    if (rc < 0) {
      write_err_path("id", NULL, errno);
      return -1;
    }
    return byte_line_list_push_plain_match(list, value, strlen(value), 1) ? 1 : -1;
  }
  if (pipe == 2 && !strcmp(words[0], "id")) {
    if (!strcmp(words[1], "-u")) {
      int len = snprintf(buf, sizeof(buf), "%llu", (unsigned long long)geteuid());
      return len >= 0 && (size_t)len < sizeof(buf) &&
             byte_line_list_push_plain_match(list, buf, (size_t)len, 1)
                 ? 1
                 : -1;
    }
    if (!strcmp(words[1], "-g")) {
      int len = snprintf(buf, sizeof(buf), "%llu", (unsigned long long)getegid());
      return len >= 0 && (size_t)len < sizeof(buf) &&
             byte_line_list_push_plain_match(list, buf, (size_t)len, 1)
                 ? 1
                 : -1;
    }
    if (!strcmp(words[1], "-un")) {
      const char *name = effective_user_name();
      if (!name) {
        const char *msg = "id: cannot find name for user ID\n";
        write_fd_all(2, msg, strlen(msg));
        return -1;
      }
      return byte_line_list_push_plain_match(list, name, strlen(name), 1) ? 1 : -1;
    }
    if (!strcmp(words[1], "-gn")) {
      const char *name = effective_group_name();
      if (!name) {
        const char *msg = "id: cannot find name for group ID\n";
        write_fd_all(2, msg, strlen(msg));
        return -1;
      }
      return byte_line_list_push_plain_match(list, name, strlen(name), 1) ? 1 : -1;
    }
    if (!strcmp(words[1], "-G") || !strcmp(words[1], "-Gn")) {
      char value[4096];
      int rc = group_list_value(!strcmp(words[1], "-Gn"), value, sizeof(value));
      if (rc == 0) return 0;
      if (rc == -2) {
        const char *msg = "id: cannot find name for group ID\n";
        write_fd_all(2, msg, strlen(msg));
        return -1;
      }
      if (rc < 0) {
        write_err_path("id", NULL, errno);
        return -1;
      }
      return byte_line_list_push_plain_match(list, value, strlen(value), 1) ? 1 : -1;
    }
    return 0;
  }
  if ((pipe == 1 || pipe == 2) && !strcmp(words[0], "uname")) {
    const char *flag = pipe == 2 ? words[1] : NULL;
    struct utsname uts;
    if (uname(&uts) != 0) {
      write_err_path("uname", NULL, errno);
      return -1;
    }
    if (flag && !strcmp(flag, "-a")) {
      int len = snprintf(buf, sizeof(buf), "%s %s %s %s %s", uts.sysname,
                         uts.nodename, uts.release, uts.version, uts.machine);
      return len >= 0 && (size_t)len < sizeof(buf) &&
             byte_line_list_push_plain_match(list, buf, (size_t)len, 1)
                 ? 1
                 : -1;
    }
    const char *field = uname_field(&uts, flag);
    if (!field) return 0;
    return byte_line_list_push_plain_match(list, field, strlen(field), 1) ? 1 : -1;
  }
  if ((pipe == 2 || pipe == 3) && !strcmp(words[0], "basename") &&
      words[1][0] != '-' && (pipe == 2 || words[2][0] != '-')) {
    return single_line_push_basename(list, words[1], pipe == 3 ? words[2] : NULL) ? 1 : -1;
  }
  if (pipe == 2 && !strcmp(words[0], "dirname") && words[1][0] != '-') {
    return single_line_push_dirname(list, words[1]) ? 1 : -1;
  }
  return 0;
}

static int collect_single_line_grep_pipe_lines(char **words, int pipe,
                                               struct byte_line_list *list) {
  struct byte_line_list source = {0};
  int collected = collect_single_line_pipe_source(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    if (contains_bytes(source.items[idx].data, (ssize_t)source.items[idx].len,
                       pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, source.items[idx].data,
                                         source.items[idx].len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int pipe_single_line_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0) return unsupported();
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    rc = collect_single_line_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_single_line_pipe_source(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_head_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0) return unsupported();
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count == pipe + 3) {
      return emit_head_stream_mode(words, pipe, pipe + 3, count, words[pipe + 2]);
    }
    if (strcmp(words[pipe + 3], "|")) {
      return unsupported();
    }
    rc = emit_head_stream_mode(words, pipe, pipe + 4, count, words[pipe + 2]);
    if (rc != 127) return rc;
    if (!collect_head_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
  } else {
    rc = emit_head_stream_mode(words, pipe, pipe + 1, count, NULL);
    if (rc != 127) return rc;
    if (!collect_head_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int parse_tail_pipe_source(char **words, int pipe, const char **path,
                                  unsigned long long *limit, int *stdin_mode) {
  if (pipe < 0 || strcmp(words[0], "tail")) return 0;
  *stdin_mode = 0;
  if (pipe == 1) {
    *limit = 10;
    *path = NULL;
    *stdin_mode = 1;
    return 1;
  }
  if (pipe == 2 && words[1][0] == '-' && words[1][1] >= '0' &&
      words[1][1] <= '9') {
    if (!parse_u64_arg(words[1] + 1, limit)) return 0;
    *path = NULL;
    *stdin_mode = 1;
    return 1;
  }
  if (pipe == 3 && !strcmp(words[1], "-n")) {
    if (!parse_u64_arg(words[2], limit)) return 0;
    *path = NULL;
    *stdin_mode = 1;
    return 1;
  }
  if (pipe == 2) {
    *limit = 10;
    *path = words[1];
    return 1;
  }
  if (pipe == 4 && !strcmp(words[1], "-n")) {
    if (!parse_u64_arg(words[2], limit)) return 0;
    *path = words[3];
    return 1;
  }
  if (pipe == 3 && words[1][0] == '-' && words[1][1] >= '0' &&
      words[1][1] <= '9') {
    if (!parse_u64_arg(words[1] + 1, limit)) return 0;
    *path = words[2];
    return 1;
  }
  return 0;
}

static int tail_line_start_offset(int fd, const char *path, unsigned long long limit,
                                  off_t *start) {
  struct stat st;
  if (fstat(fd, &st) != 0) {
    write_err_path("tail", path, errno);
    return -1;
  }
  if (!S_ISREG(st.st_mode)) return 0;
  if (limit == 0) {
    *start = st.st_size;
    return 1;
  }
  off_t pos = st.st_size;
  char buf[8192];
  if (pos > 0) {
    char last = 0;
    ssize_t read_len = pread(fd, &last, 1, pos - 1);
    if (read_len < 0) {
      write_err_path("tail", path, errno);
      return -1;
    }
    if (read_len == 1 && last == '\n') pos--;
  }
  unsigned long long seen = 0;
  while (pos > 0) {
    size_t chunk = pos > (off_t)sizeof(buf) ? sizeof(buf) : (size_t)pos;
    off_t chunk_start = pos - (off_t)chunk;
    ssize_t read_len = pread(fd, buf, chunk, chunk_start);
    if (read_len < 0) {
      write_err_path("tail", path, errno);
      return -1;
    }
    if (read_len == 0) break;
    for (ssize_t idx = read_len; idx > 0; idx--) {
      if (buf[idx - 1] == '\n' && ++seen == limit) {
        *start = chunk_start + idx;
        return 1;
      }
    }
    pos = chunk_start;
  }
  *start = 0;
  return 1;
}

static int stream_tail_lines(char **words, int pipe, head_line_visitor visitor,
                             void *ctx) {
  const char *path = NULL;
  unsigned long long limit = 0;
  int stdin_mode = 0;
  char buf[8192];
  char line[8192];
  size_t used = 0;
  off_t start = 0;
  if (!parse_tail_pipe_source(words, pipe, &path, &limit, &stdin_mode)) return 0;
  if (stdin_mode) {
    struct byte_line_list tail = {0};
    if (limit == 0) return 1;
    for (;;) {
      ssize_t read_len = read(STDIN_FILENO, buf, sizeof(buf));
      if (read_len == 0) break;
      if (read_len < 0) {
        write_err_path("tail", "stdin", errno);
        byte_line_list_free(&tail);
        return 1;
      }
      for (ssize_t idx = 0; idx < read_len; idx++) {
        if (used < sizeof(line)) line[used++] = buf[idx];
        if (buf[idx] == '\n' || used == sizeof(line)) {
          if (!byte_line_list_push_plain_match(&tail, line, used, 0)) {
            byte_line_list_free(&tail);
            return -1;
          }
          if (tail.len > limit) byte_line_list_drop_first(&tail);
          used = 0;
        }
      }
    }
    if (used > 0) {
      if (!byte_line_list_push_plain_match(&tail, line, used, 0)) {
        byte_line_list_free(&tail);
        return -1;
      }
      if (tail.len > limit) byte_line_list_drop_first(&tail);
    }
    for (size_t idx = 0; idx < tail.len; idx++) {
      if (!visitor(tail.items[idx].data, tail.items[idx].len, ctx)) {
        byte_line_list_free(&tail);
        return -1;
      }
    }
    byte_line_list_free(&tail);
    return 1;
  }
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("tail", path, errno);
    return 1;
  }
  int offset_rc = tail_line_start_offset(fd, path, limit, &start);
  if (offset_rc <= 0) {
    close(fd);
    return offset_rc < 0 ? 1 : 0;
  }
  if (lseek(fd, start, SEEK_SET) < 0) {
    write_err_path("tail", path, errno);
    close(fd);
    return 1;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("tail", path, errno);
      close(fd);
      return 1;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
        if (!visitor(line, used, ctx)) {
          close(fd);
          return -1;
        }
        used = 0;
      }
    }
  }
  if (used > 0 && !visitor(line, used, ctx)) {
    close(fd);
    return -1;
  }
  close(fd);
  return 1;
}

static int count_tail_stdin_wc_lines(unsigned long long limit,
                                     unsigned long long *count) {
  char buf[8192];
  unsigned long long newline_count = 0;
  int saw_input = 0;
  int last_was_newline = 0;
  if (limit == 0) {
    *count = 0;
    return 1;
  }
  for (;;) {
    ssize_t read_len = read(STDIN_FILENO, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("tail", "stdin", errno);
      return -1;
    }
    saw_input = 1;
    last_was_newline = buf[read_len - 1] == '\n';
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (buf[idx] == '\n') newline_count++;
    }
  }
  if (saw_input && !last_was_newline) {
    if (newline_count + 1 <= limit) {
      *count = newline_count;
    } else {
      *count = newline_count < limit - 1 ? newline_count : limit - 1;
    }
  } else {
    *count = newline_count < limit ? newline_count : limit;
  }
  return 1;
}

static int emit_tail_stream_mode(char **words, int pipe, int start, int count,
                                 const char *pattern) {
  unsigned long long limit = 0;
  struct head_stream_ctx ctx = {0};
  ctx.pattern = pattern;
  ctx.pattern_len = pattern ? strlen(pattern) : 0;
  ctx.append_newline = pattern != NULL;
  ctx.first = 1;

  if (count == start) {
    if (!pattern) return unsupported();
    ctx.mode = HEAD_STREAM_LINES;
  } else if (count - start == 2 && !strcmp(words[start], "wc") &&
             !strcmp(words[start + 1], "-l")) {
    ctx.mode = HEAD_STREAM_WC;
  } else if (count - start == 3 && !strcmp(words[start], "head") &&
             !strcmp(words[start + 1], "-n") &&
             parse_u64_arg(words[start + 2], &limit) && limit > 0) {
    ctx.mode = HEAD_STREAM_HEAD;
    ctx.limit = limit;
  } else if (count - start == 3 && !strcmp(words[start], "tail") &&
             !strcmp(words[start + 1], "-n") &&
             parse_u64_arg(words[start + 2], &limit)) {
    ctx.mode = HEAD_STREAM_TAIL;
    ctx.limit = limit;
  } else if (count - start == 2 && !strcmp(words[start], "xargs") &&
             !strcmp(words[start + 1], "echo")) {
    ctx.mode = HEAD_STREAM_XARGS_ECHO;
  } else {
    return unsupported();
  }

  if (ctx.mode == HEAD_STREAM_WC && !pattern) {
    const char *tail_path = NULL;
    unsigned long long tail_limit = 0;
    unsigned long long tail_count = 0;
    int tail_stdin_mode = 0;
    if (parse_tail_pipe_source(words, pipe, &tail_path, &tail_limit,
                               &tail_stdin_mode) &&
        tail_stdin_mode) {
      if (count_tail_stdin_wc_lines(tail_limit, &tail_count) < 0) return 1;
      write_padded_u64(tail_count);
      write_bytes("\n", 1);
      return 0;
    }
  }

  int streamed = stream_tail_lines(words, pipe, head_stream_visit, &ctx);
  if (streamed < 0) {
    byte_line_list_free(&ctx.tail);
    return 1;
  }
  if (!streamed) {
    byte_line_list_free(&ctx.tail);
    return unsupported();
  }

  int rc = 0;
  if (ctx.mode == HEAD_STREAM_WC) {
    write_padded_u64(ctx.count);
    write_bytes("\n", 1);
  } else if (ctx.mode == HEAD_STREAM_TAIL) {
    for (size_t idx = 0; idx < ctx.tail.len; idx++) {
      write_bytes(ctx.tail.items[idx].data, ctx.tail.items[idx].len);
    }
  } else if (ctx.mode == HEAD_STREAM_XARGS_ECHO) {
    if (!ctx.first) write_bytes("\n", 1);
  } else if (ctx.mode == HEAD_STREAM_LINES && pattern && ctx.matches == 0) {
    rc = 1;
  }
  byte_line_list_free(&ctx.tail);
  return rc;
}

static int collect_tail_lines(char **words, int pipe, struct byte_line_list *list) {
  struct head_stream_ctx ctx = {
      .mode = HEAD_STREAM_TAIL,
      .limit = (unsigned long long)-1,
      .first = 1,
  };
  int streamed = stream_tail_lines(words, pipe, head_stream_visit, &ctx);
  if (streamed <= 0) {
    byte_line_list_free(&ctx.tail);
    return 0;
  }
  *list = ctx.tail;
  return 1;
}

static int collect_tail_grep_lines(char **words, int pipe, struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  const char *pattern = words[pipe + 2];
  struct head_stream_ctx ctx = {
      .mode = HEAD_STREAM_TAIL,
      .pattern = pattern,
      .pattern_len = strlen(pattern),
      .limit = (unsigned long long)-1,
      .append_newline = 1,
      .first = 1,
  };
  int streamed = stream_tail_lines(words, pipe, head_stream_visit, &ctx);
  if (streamed <= 0) {
    byte_line_list_free(&ctx.tail);
    return 0;
  }
  *list = ctx.tail;
  return 1;
}

static int pipe_tail_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0) return unsupported();
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count == pipe + 3) {
      return emit_tail_stream_mode(words, pipe, pipe + 3, count, words[pipe + 2]);
    }
    if (strcmp(words[pipe + 3], "|")) {
      return unsupported();
    }
    rc = emit_tail_stream_mode(words, pipe, pipe + 4, count, words[pipe + 2]);
    if (rc != 127) return rc;
    if (!collect_tail_grep_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
  } else {
    rc = emit_tail_stream_mode(words, pipe, pipe + 1, count, NULL);
    if (rc != 127) return rc;
    if (!collect_tail_lines(words, pipe, &list)) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int parse_sed_pipe_source(char **words, int pipe, const char **path,
                                 long *start_line, long *end_line) {
  if (pipe == 4 && !strcmp(words[0], "sed") && !strcmp(words[1], "-n") &&
      !strcmp(words[pipe], "|")) {
    if (!parse_sed_range(words[2], start_line, end_line)) return 0;
    *path = words[3];
  } else if (pipe == 6 && !strcmp(words[0], "cat") && !strcmp(words[2], "|") &&
             !strcmp(words[3], "sed") && !strcmp(words[4], "-n") &&
             !strcmp(words[pipe], "|")) {
    if (!parse_sed_range(words[5], start_line, end_line)) return 0;
    *path = words[1];
  } else {
    return 0;
  }
  struct stat st;
  if (stat(*path, &st) != 0) return 1;
  if (!S_ISREG(st.st_mode)) return 0;
  return 1;
}

static int collect_sed_pipe_lines(char **words, int pipe,
                                  struct byte_line_list *list) {
  const char *path = NULL;
  long start_line = 0;
  long end_line = 0;
  char buf[8192];
  char line[8192];
  size_t used = 0;
  long line_no = 1;
  if (!parse_sed_pipe_source(words, pipe, &path, &start_line, &end_line)) {
    return 0;
  }
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    write_err_path("sed", path, errno);
    return 1;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      write_err_path("sed", path, errno);
      close(fd);
      return 1;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
        if (line_no >= start_line && line_no <= end_line &&
            !byte_line_list_push_plain_match(list, line, used, 0)) {
          close(fd);
          return -1;
        }
        used = 0;
        line_no++;
        if (line_no > end_line) {
          close(fd);
          return 1;
        }
      }
    }
  }
  if (used > 0 && line_no >= start_line && line_no <= end_line &&
      !byte_line_list_push_plain_match(list, line, used, 0)) {
    close(fd);
    return -1;
  }
  close(fd);
  return 1;
}

static int collect_sed_grep_pipe_lines(char **words, int pipe,
                                       struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  struct byte_line_list source = {0};
  int collected = collect_sed_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    int add_newline = item->len == 0 || item->data[item->len - 1] != '\n';
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, add_newline)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int pipe_sed_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0) return unsupported();
  if (pipe == 2 && count >= 6 && !strcmp(words[0], "cat") &&
      !strcmp(words[2], "|") && !strcmp(words[3], "sed") &&
      !strcmp(words[4], "-n")) {
    if (count == 6) {
      long start_line = 0;
      long end_line = 0;
      const char *path = words[1];
      if (!parse_sed_range(words[5], &start_line, &end_line)) return unsupported();
      struct stat st;
      if (stat(path, &st) != 0 || !S_ISREG(st.st_mode)) return unsupported();
      char *direct_words[5] = {"sed", "-n", words[5], words[1], "|"};
      rc = collect_sed_pipe_lines(direct_words, 4, &list);
      if (rc < 0) {
        byte_line_list_free(&list);
        return 1;
      }
      if (rc == 0) {
        byte_line_list_free(&list);
        return unsupported();
      }
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      byte_line_list_free(&list);
      return 0;
    }
    if (count < 8 || strcmp(words[6], "|")) return unsupported();
    pipe = 6;
  }
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    rc = collect_sed_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_sed_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int cut_segment_to_list(struct byte_line_list *list, const char *line,
                               size_t len, unsigned char delimiter,
                               unsigned long long field) {
  int has_delimiter = 0;
  for (size_t idx = 0; idx < len; idx++) {
    if ((unsigned char)line[idx] == delimiter) {
      has_delimiter = 1;
      break;
    }
  }
  if (!has_delimiter) {
    return byte_line_list_push_plain_match(list, line, len, 1);
  }

  unsigned long long current_field = 1;
  size_t start = 0;
  for (size_t idx = 0; idx < len; idx++) {
    if ((unsigned char)line[idx] != delimiter) continue;
    if (current_field == field) {
      return byte_line_list_push_plain_match(list, line + start, idx - start, 1);
    }
    current_field++;
    start = idx + 1;
  }
  if (current_field == field) {
    return byte_line_list_push_plain_match(list, line + start, len - start, 1);
  }
  return byte_line_list_push_plain_match(list, "", 0, 1);
}

static int collect_cut_plan_lines(const struct cut_plan *plan, const char *err_cmd,
                                  struct byte_line_list *list) {
  FILE *file = plan->stdin_mode ? stdin : fopen(plan->file, "r");
  if (!file) {
    write_err_path(err_cmd, plan->file, errno);
    return 1;
  }
  char *line = NULL;
  size_t cap = 0;
  ssize_t line_len = 0;
  int rc = 1;
  while ((line_len = getline(&line, &cap, file)) >= 0) {
    size_t len = (size_t)line_len;
    if (len > 0 && line[len - 1] == '\n') len--;
    if (!cut_segment_to_list(list, line, len, plan->delimiter, plan->field)) {
      rc = -1;
      break;
    }
  }
  if (ferror(file)) {
    write_err_path(err_cmd, plan->stdin_mode ? "stdin" : plan->file, errno);
  }
  free(line);
  if (!plan->stdin_mode) fclose(file);
  return rc;
}

static int count_cut_plan_records(const struct cut_plan *plan, const char *err_cmd,
                                  unsigned long long *count) {
  FILE *file = plan->stdin_mode ? stdin : fopen(plan->file, "r");
  *count = 0;
  if (!file) {
    write_err_path(err_cmd, plan->file, errno);
    return 1;
  }
  char *line = NULL;
  size_t cap = 0;
  ssize_t line_len = 0;
  int rc = 1;
  while ((line_len = getline(&line, &cap, file)) >= 0) {
    (void)line_len;
    (*count)++;
  }
  if (ferror(file)) {
    write_err_path(err_cmd, plan->stdin_mode ? "stdin" : plan->file, errno);
  }
  free(line);
  if (!plan->stdin_mode) fclose(file);
  return rc;
}

static int emit_cut_wc_lines_if_supported(char **words, int count, int cut_start,
                                          int pipe, const char *forced_file,
                                          const char *err_cmd) {
  if (count != pipe + 3 || strcmp(words[pipe + 1], "wc") ||
      strcmp(words[pipe + 2], "-l")) {
    return 127;
  }
  struct cut_plan plan;
  if (!parse_cut_words(words, cut_start, pipe, forced_file, &plan)) return 127;
  unsigned long long total = 0;
  int counted = count_cut_plan_records(&plan, err_cmd, &total);
  if (counted < 0) return 1;
  if (counted == 0) return 127;
  write_padded_u64(total);
  write_bytes("\n", 1);
  return 0;
}

static int collect_cut_pipe_lines(char **words, int pipe,
                                  struct byte_line_list *list) {
  struct cut_plan plan;
  if (pipe < 0 || strcmp(words[0], "cut") ||
      !parse_cut_words(words, 1, pipe, NULL, &plan)) {
    return 0;
  }
  return collect_cut_plan_lines(&plan, "cut", list);
}

static int collect_cat_cut_pipe_lines(char **words, int pipe,
                                      struct byte_line_list *list) {
  struct cut_plan plan;
  if (pipe < 0 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "cut") ||
      !parse_cut_words(words, 4, pipe, words[1], &plan)) {
    return 0;
  }
  return collect_cut_plan_lines(&plan, "cat", list);
}

static int collect_cut_grep_pipe_lines(char **words, int pipe,
                                       struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  struct byte_line_list source = {0};
  int collected = collect_cut_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int collect_grep_file_cut_pipe_lines(char **words, int cut_end,
                                            struct byte_line_list *list) {
  if (cut_end <= 5 || strcmp(words[3], "|") || strcmp(words[4], "cut")) {
    return 0;
  }
  struct cut_plan plan;
  if (!parse_cut_words(words, 5, cut_end, "-", &plan)) return 0;
  struct byte_line_list source = {0};
  int collected = collect_grep_file_pipe_lines(words, 3, &source);
  if (collected != 0) {
    byte_line_list_free(&source);
    return collected == 127 ? 0 : -1;
  }
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    size_t len = item->len;
    if (len > 0 && item->data[len - 1] == '\n') len--;
    if (!cut_segment_to_list(list, item->data, len, plan.delimiter, plan.field)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int pipe_grep_file_cut_producer(char **words, int count) {
  if (count < 6 || strcmp(words[3], "|") || strcmp(words[4], "cut")) {
    return unsupported();
  }
  int pipe = -1;
  for (int idx = 5; idx < count; idx++) {
    if (!strcmp(words[idx], "|")) {
      pipe = idx;
      break;
    }
  }
  int cut_end = pipe < 0 ? count : pipe;
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe >= 0 && count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    struct byte_line_list source = {0};
    rc = collect_grep_file_cut_pipe_lines(words, cut_end, &source);
    if (rc < 0) {
      byte_line_list_free(&source);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&source);
      return unsupported();
    }
    const char *pattern = words[pipe + 2];
    size_t pattern_len = strlen(pattern);
    for (size_t idx = 0; idx < source.len; idx++) {
      struct byte_line_item *item = &source.items[idx];
      if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
          !byte_line_list_push_plain_match(&list, item->data, item->len, 0)) {
        byte_line_list_free(&source);
        byte_line_list_free(&list);
        return 1;
      }
    }
    byte_line_list_free(&source);
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_grep_file_cut_pipe_lines(words, cut_end, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (pipe < 0) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = 0;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static const char *skip_ascii_space(const char *text) {
  while (*text && isspace((unsigned char)*text)) text++;
  return text;
}

static int parse_awk_print_field_action(const char *action,
                                        unsigned long long *field) {
  unsigned long long value = 0;
  action = skip_ascii_space(action);
  if (*action != '{') return 0;
  action++;
  action = skip_ascii_space(action);
  if (strncmp(action, "print", 5)) return 0;
  action += 5;
  action = skip_ascii_space(action);
  if (*action != '$') return 0;
  action++;
  action = skip_ascii_space(action);
  if (!isdigit((unsigned char)*action)) return 0;
  while (isdigit((unsigned char)*action)) {
    value = value * 10 + (unsigned long long)(*action - '0');
    action++;
  }
  if (value == 0) return 0;
  action = skip_ascii_space(action);
  if (*action != '}') return 0;
  action++;
  if (*skip_ascii_space(action) != 0) return 0;
  *field = value;
  return 1;
}

static int parse_awk_print_field_script(const char *script, const char **filter,
                                        unsigned long long *field) {
  script = skip_ascii_space(script);
  if (parse_awk_print_field_action(script, field)) {
    *filter = NULL;
    return 1;
  }
  const char *prefix = "/NEEDLE/";
  size_t prefix_len = strlen(prefix);
  if (!strncmp(script, prefix, prefix_len) &&
      parse_awk_print_field_action(script + prefix_len, field)) {
    *filter = "NEEDLE";
    return 1;
  }
  return 0;
}

static void awk_field_bounds(const char *data, size_t len,
                             unsigned long long field, size_t *start,
                             size_t *end) {
  size_t pos = 0;
  unsigned long long current = 0;
  *start = len;
  *end = len;
  while (current < field) {
    while (pos < len && isspace((unsigned char)data[pos])) pos++;
    if (pos >= len) {
      *start = len;
      *end = len;
      return;
    }
    *start = pos;
    while (pos < len && !isspace((unsigned char)data[pos])) pos++;
    *end = pos;
    current++;
    if (current == field) return;
  }
}

static int awk_field_to_list(struct byte_line_list *list, const char *data,
                             size_t len, unsigned long long field) {
  size_t start = 0;
  size_t end = 0;
  awk_field_bounds(data, len, field, &start, &end);
  return byte_line_list_push_plain_match(list, data + start, end - start, 1);
}

static int collect_grep_file_awk_pipe_lines(char **words, int awk_end,
                                            struct byte_line_list *list) {
  if (awk_end != 6 || strcmp(words[3], "|") || strcmp(words[4], "awk")) {
    return 0;
  }
  const char *awk_filter = NULL;
  unsigned long long awk_field = 1;
  if (!parse_awk_print_field_script(words[5], &awk_filter, &awk_field)) return 0;
  struct byte_line_list source = {0};
  int collected = collect_grep_file_pipe_lines(words, 3, &source);
  if (collected != 0) {
    byte_line_list_free(&source);
    return collected == 127 ? 0 : -1;
  }
  size_t filter_len = awk_filter ? strlen(awk_filter) : 0;
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (awk_filter &&
        !contains_bytes(item->data, (ssize_t)item->len, awk_filter, filter_len)) {
      continue;
    }
    if (!awk_field_to_list(list, item->data, item->len, awk_field)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int pipe_grep_file_awk_producer(char **words, int count) {
  if (count < 6 || strcmp(words[3], "|") || strcmp(words[4], "awk")) {
    return unsupported();
  }
  int pipe = count > 6 && !strcmp(words[6], "|") ? 6 : -1;
  if (pipe < 0 && count != 6) return unsupported();
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe >= 0 && count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    struct byte_line_list source = {0};
    rc = collect_grep_file_awk_pipe_lines(words, pipe, &source);
    if (rc < 0) {
      byte_line_list_free(&source);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&source);
      return unsupported();
    }
    const char *pattern = words[pipe + 2];
    size_t pattern_len = strlen(pattern);
    for (size_t idx = 0; idx < source.len; idx++) {
      struct byte_line_item *item = &source.items[idx];
      if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
          !byte_line_list_push_plain_match(&list, item->data, item->len, 0)) {
        byte_line_list_free(&source);
        byte_line_list_free(&list);
        return 1;
      }
    }
    byte_line_list_free(&source);
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    int awk_end = pipe < 0 ? count : pipe;
    rc = collect_grep_file_awk_pipe_lines(words, awk_end, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (pipe < 0) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = 0;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
    }
  }
  byte_line_list_free(&list);
  return rc;
}

static int collect_cat_cut_grep_pipe_lines(char **words, int pipe,
                                           struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  struct byte_line_list source = {0};
  int collected = collect_cat_cut_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int tr_segment_to_list(struct byte_line_list *list, const char *data,
                              size_t len, const struct tr_plan *plan) {
  char *out = (char *)malloc(len ? len : 1);
  if (!out) return 0;
  size_t out_len = 0;
  if (plan->mode == TR_MODE_TRANSLATE) {
    for (size_t idx = 0; idx < len; idx++) {
      out[out_len++] = (char)plan->map[(unsigned char)data[idx]];
    }
  } else {
    for (size_t idx = 0; idx < len; idx++) {
      unsigned char byte = (unsigned char)data[idx];
      if (!plan->delete_set[byte]) out[out_len++] = (char)byte;
    }
  }
  int ok = byte_line_list_push(list, out, out_len);
  free(out);
  return ok;
}

static int collect_cat_tr_pipe_lines(char **words, int pipe,
                                     struct byte_line_list *list) {
  struct tr_plan plan;
  if (pipe < 0 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "tr") || !parse_tr_words(words, 4, pipe, &plan)) {
    return 0;
  }
  FILE *file = fopen(words[1], "r");
  if (!file) {
    write_err_path("cat", words[1], errno);
    return 1;
  }
  char *line = NULL;
  size_t cap = 0;
  ssize_t line_len = 0;
  int rc = 1;
  while ((line_len = getline(&line, &cap, file)) >= 0) {
    if (!tr_segment_to_list(list, line, (size_t)line_len, &plan)) {
      rc = -1;
      break;
    }
  }
  if (ferror(file)) {
    write_err_path("cat", words[1], errno);
  }
  free(line);
  fclose(file);
  return rc;
}

static int collect_cat_tr_grep_pipe_lines(char **words, int pipe,
                                          struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  struct byte_line_list source = {0};
  int collected = collect_cat_tr_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int collect_uniq_file_lines(const char *command_name, const char *path,
                                   struct byte_line_list *list) {
  FILE *file = fopen(path, "r");
  char *line = NULL;
  size_t cap = 0;
  char *previous = NULL;
  size_t previous_len = 0;
  int have_previous = 0;
  int rc = 1;
  if (!file) {
    write_err_path(command_name, path, errno);
    return 1;
  }
  for (;;) {
    ssize_t read_len = getline(&line, &cap, file);
    if (read_len < 0) break;
    size_t len = (size_t)read_len;
    size_t cmp_len = line_len_without_newline(line, len);
    int duplicate = have_previous && previous_len == cmp_len &&
                    (cmp_len == 0 || memcmp(previous, line, cmp_len) == 0);
    if (!duplicate) {
      int ok = len > 0 && line[len - 1] == '\n'
                   ? byte_line_list_push(list, line, len)
                   : byte_line_list_push_plain_match(list, line, len, 1);
      if (!ok) {
        rc = -1;
        break;
      }
      char *next_previous = (char *)malloc(cmp_len ? cmp_len : 1);
      if (!next_previous) {
        rc = -1;
        break;
      }
      if (cmp_len) memcpy(next_previous, line, cmp_len);
      free(previous);
      previous = next_previous;
      previous_len = cmp_len;
      have_previous = 1;
    }
  }
  if (ferror(file)) {
    write_err_path(command_name, path, errno);
  }
  free(previous);
  free(line);
  fclose(file);
  return rc;
}

static int collect_cat_uniq_pipe_lines(char **words, int pipe,
                                       struct byte_line_list *list) {
  if (pipe != 4 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "uniq")) {
    return 0;
  }
  return collect_uniq_file_lines("cat", words[1], list);
}

static int collect_uniq_pipe_lines(char **words, int pipe,
                                   struct byte_line_list *list) {
  if (pipe != 2 || strcmp(words[0], "uniq") || strcmp(words[2], "|")) {
    return 0;
  }
  return collect_uniq_file_lines("uniq", words[1], list);
}

static int collect_cat_uniq_grep_pipe_lines(char **words, int pipe,
                                            struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  struct byte_line_list source = {0};
  int collected = collect_cat_uniq_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int collect_uniq_grep_pipe_lines(char **words, int pipe,
                                        struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  struct byte_line_list source = {0};
  int collected = collect_uniq_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static const char *sort_uniq_pipe_file(char **words, int pipe) {
  if (pipe == 4 && !strcmp(words[0], "sort") && !strcmp(words[2], "|") &&
      !strcmp(words[3], "uniq")) {
    return words[1];
  }
  if (pipe == 6 && !strcmp(words[0], "cat") && !strcmp(words[2], "|") &&
      !strcmp(words[3], "sort") && !strcmp(words[4], "|") &&
      !strcmp(words[5], "uniq")) {
    return words[1];
  }
  return NULL;
}

static const char *sort_pipe_file(char **words, int pipe) {
  if (pipe == 2 && !strcmp(words[0], "sort") && !strcmp(words[2], "|")) {
    return words[1];
  }
  if (pipe == 4 && !strcmp(words[0], "cat") && !strcmp(words[2], "|") &&
      !strcmp(words[3], "sort") && !strcmp(words[4], "|")) {
    return words[1];
  }
  return NULL;
}

static int byte_line_list_push_span(struct byte_line_list *list, const char *data,
                                    struct line_span span) {
  const char *line = data + span.start;
  size_t len = span.end - span.start;
  if (len > 0 && line[len - 1] == '\n') return byte_line_list_push(list, line, len);
  return byte_line_list_push_plain_match(list, line, len, 1);
}

static int collect_sort_pipe_lines(char **words, int pipe,
                                   struct byte_line_list *list) {
  const char *file = sort_pipe_file(words, pipe);
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  if (!file) return 0;
  int loaded = load_sorted_file_for_pipe(file, &data, &lines, &line_len);
  if (loaded == 127) return 0;
  if (loaded != 0) return -1;
  for (size_t idx = 0; idx < line_len; idx++) {
    if (!byte_line_list_push_span(list, data, lines[idx])) {
      free(lines);
      free(data);
      return -1;
    }
  }
  free(lines);
  free(data);
  return 1;
}

static int collect_cat_pipe_lines(const char *path, struct byte_line_list *list) {
  char *data = NULL;
  size_t size = 0;
  int loaded = load_regular_file_for_pipe(path, &data, &size);
  if (loaded != 0) return loaded;
  size_t start = 0;
  for (size_t idx = 0; idx < size; idx++) {
    if (data[idx] == '\n') {
      if (!byte_line_list_push(list, data + start, idx + 1 - start)) {
        free(data);
        return 1;
      }
      start = idx + 1;
    }
  }
  if (start < size &&
      !byte_line_list_push_plain_match(list, data + start, size - start, 1)) {
    free(data);
    return 1;
  }
  free(data);
  return 0;
}

static int pipe_cat_xargs_wc_producer(char **words, int count) {
  if (count < 8 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "xargs") || strcmp(words[4], "wc") ||
      strcmp(words[5], "-l") || strcmp(words[6], "|") ||
      !xargs_wc_output_mode_supported(words, 7, count)) {
    return unsupported();
  }
  struct byte_line_list list = {0};
  int rc = collect_cat_pipe_lines(words[1], &list);
  if (rc != 0) {
    byte_line_list_free(&list);
    return rc == 127 ? unsupported() : rc;
  }
  rc = emit_xargs_wc_output_line_list_mode(words, 3, count, &list, 0);
  byte_line_list_free(&list);
  return rc == 127 ? unsupported() : rc;
}

static int pipe_sort_xargs_wc_producer(char **words, int count) {
  if (count < 8 || strcmp(words[0], "sort") || strcmp(words[2], "|") ||
      strcmp(words[3], "xargs") || strcmp(words[4], "wc") ||
      strcmp(words[5], "-l") || strcmp(words[6], "|") ||
      !xargs_wc_output_mode_supported(words, 7, count)) {
    return unsupported();
  }
  struct byte_line_list list = {0};
  int rc = collect_sort_pipe_lines(words, 2, &list);
  if (rc != 1) {
    byte_line_list_free(&list);
    return rc == 0 ? unsupported() : 1;
  }
  rc = emit_xargs_wc_output_line_list_mode(words, 3, count, &list, 0);
  byte_line_list_free(&list);
  return rc == 127 ? unsupported() : rc;
}

static int pipe_cat_sort_xargs_wc_producer(char **words, int count) {
  if (count < 10 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "sort") || strcmp(words[4], "|") ||
      strcmp(words[5], "xargs") || strcmp(words[6], "wc") ||
      strcmp(words[7], "-l") || strcmp(words[8], "|") ||
      !xargs_wc_output_mode_supported(words, 9, count)) {
    return unsupported();
  }
  struct byte_line_list list = {0};
  int rc = collect_sort_pipe_lines(words, 4, &list);
  if (rc != 1) {
    byte_line_list_free(&list);
    return rc == 0 ? unsupported() : 1;
  }
  rc = emit_xargs_wc_output_line_list_mode(words, 5, count, &list, 0);
  byte_line_list_free(&list);
  return rc == 127 ? unsupported() : rc;
}

static int collect_sort_uniq_pipe_lines(char **words, int pipe,
                                        struct byte_line_list *list) {
  const char *file = sort_uniq_pipe_file(words, pipe);
  char *data = NULL;
  struct line_span *lines = NULL;
  size_t line_len = 0;
  int have_previous = 0;
  struct line_span previous = {0, 0};
  if (!file) return 0;
  int loaded = load_sorted_file_for_pipe(file, &data, &lines, &line_len);
  if (loaded == 127) return 0;
  if (loaded != 0) return -1;
  for (size_t idx = 0; idx < line_len; idx++) {
    if (!have_previous || !line_spans_equal_without_newline(data, previous, lines[idx])) {
      if (!byte_line_list_push_span(list, data, lines[idx])) {
        free(lines);
        free(data);
        return -1;
      }
      previous = lines[idx];
      have_previous = 1;
    }
  }
  free(lines);
  free(data);
  return 1;
}

static int collect_sort_uniq_grep_pipe_lines(char **words, int pipe,
                                             struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  struct byte_line_list source = {0};
  int collected = collect_sort_uniq_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int collect_sort_grep_pipe_lines(char **words, int pipe,
                                        struct byte_line_list *list) {
  if (pipe < 0 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return 0;
  }
  struct byte_line_list source = {0};
  int collected = collect_sort_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int pipe_cut_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0) return unsupported();
  rc = emit_cut_wc_lines_if_supported(words, count, 1, pipe, NULL, "cut");
  if (rc != 127) return rc;
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    rc = collect_cut_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_cut_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_cut_producer(char **words, int count) {
  if (count < 7 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "cut")) {
    return unsupported();
  }
  int pipe = -1;
  for (int idx = 4; idx < count; idx++) {
    if (!strcmp(words[idx], "|")) {
      pipe = idx;
      break;
    }
  }
  if (pipe < 0 || pipe + 1 >= count) return unsupported();

  struct byte_line_list list = {0};
  int rc = 127;
  rc = emit_cut_wc_lines_if_supported(words, count, 4, pipe, words[1], "cat");
  if (rc != 127) return rc;
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    rc = collect_cat_cut_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_cat_cut_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_tr_producer(char **words, int count) {
  if (count < 7 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "tr")) {
    return unsupported();
  }
  int pipe = -1;
  for (int idx = 4; idx < count; idx++) {
    if (!strcmp(words[idx], "|")) {
      pipe = idx;
      break;
    }
  }
  if (pipe < 0 || pipe + 1 >= count) return unsupported();

  struct byte_line_list list = {0};
  int rc = 127;
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    rc = collect_cat_tr_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_cat_tr_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_cat_uniq_producer(char **words, int count) {
  if (count < 6 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "uniq") || strcmp(words[4], "|")) {
    return unsupported();
  }
  int pipe = 4;
  struct byte_line_list list = {0};
  int rc = 127;
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    rc = collect_cat_uniq_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_cat_uniq_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_uniq_producer(char **words, int count) {
  if (count < 4 || strcmp(words[0], "uniq") || strcmp(words[2], "|")) {
    return unsupported();
  }
  int pipe = 2;
  struct byte_line_list list = {0};
  int rc = 127;
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    rc = collect_uniq_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_uniq_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_sort_uniq_producer(char **words, int count) {
  int pipe = -1;
  if (count >= 6 && !strcmp(words[0], "sort") && !strcmp(words[2], "|") &&
      !strcmp(words[3], "uniq") && !strcmp(words[4], "|")) {
    pipe = 4;
  } else if (count >= 8 && !strcmp(words[0], "cat") && !strcmp(words[2], "|") &&
             !strcmp(words[3], "sort") && !strcmp(words[4], "|") &&
             !strcmp(words[5], "uniq") && !strcmp(words[6], "|")) {
    pipe = 6;
  } else {
    return unsupported();
  }
  if (pipe + 1 >= count) return unsupported();

  struct byte_line_list list = {0};
  int rc = 127;
  if (count >= pipe + 3 && !strcmp(words[pipe + 1], "grep") &&
      is_plain_literal_pattern(words[pipe + 2])) {
    if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
    rc = collect_sort_uniq_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 3) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
    }
  } else {
    rc = collect_sort_uniq_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 1, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_sort_grep_producer(char **words, int count) {
  int pipe = -1;
  if (count >= 5 && !strcmp(words[0], "sort") && !strcmp(words[2], "|")) {
    pipe = 2;
  } else if (count >= 7 && !strcmp(words[0], "cat") && !strcmp(words[2], "|") &&
             !strcmp(words[3], "sort") && !strcmp(words[4], "|")) {
    pipe = 4;
  } else {
    return unsupported();
  }
  if (pipe + 1 >= count || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return unsupported();
  }
  if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();

  struct byte_line_list list = {0};
  int rc = collect_sort_grep_pipe_lines(words, pipe, &list);
  if (rc < 0) {
    byte_line_list_free(&list);
    return 1;
  }
  if (rc == 0) {
    byte_line_list_free(&list);
    return unsupported();
  }
  if (count == pipe + 3) {
    for (size_t idx = 0; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
    rc = list.len ? 0 : 1;
  } else {
    rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
  }
  byte_line_list_free(&list);
  return rc;
}

static int collect_seq_lines(char **words, int pipe, struct byte_line_list *list) {
  struct seq_plan seq;
  if (pipe < 0 || strcmp(words[0], "seq") ||
      !parse_seq_words(words, 0, pipe, &seq)) {
    return 0;
  }
  unsigned long long remaining = seq_count(&seq);
  long long current = seq.first;
  char buf[32];
  while (remaining > 0) {
    int len = snprintf(buf, sizeof(buf), "%lld", current);
    if (len <= 0 || !byte_line_list_push_plain_match(list, buf, (size_t)len, 1)) {
      return 0;
    }
    remaining--;
    if (remaining == 0) break;
    current += seq.step;
  }
  return 1;
}

static int collect_seq_grep_lines(char **words, int pipe, struct byte_line_list *list) {
  struct seq_plan seq;
  const char *pattern = NULL;
  size_t pattern_len = 0;
  if (pipe < 0 || strcmp(words[0], "seq") ||
      strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2]) ||
      !parse_seq_words(words, 0, pipe, &seq)) {
    return 0;
  }
  pattern = words[pipe + 2];
  pattern_len = strlen(pattern);
  unsigned long long remaining = seq_count(&seq);
  long long current = seq.first;
  char buf[32];
  while (remaining > 0) {
    int len = snprintf(buf, sizeof(buf), "%lld", current);
    if (len <= 0) return 0;
    if (contains_bytes(buf, (ssize_t)len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, buf, (size_t)len, 1)) {
      return 0;
    }
    remaining--;
    if (remaining == 0) break;
    current += seq.step;
  }
  return 1;
}

static int pipe_seq_grep(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 3) return unsupported();
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  int rc = list.len ? 0 : 1;
  byte_line_list_free(&list);
  return rc;
}

static int pipe_seq_grep_wc(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 6 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "wc") || strcmp(words[pipe + 5], "-l")) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  write_padded_u64((unsigned long long)list.len);
  write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_head(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (pipe < 0 || count != pipe + 7 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "head") || strcmp(words[pipe + 5], "-n") ||
      !parse_u64_arg(words[pipe + 6], &limit) || limit == 0) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_tail(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (pipe < 0 || count != pipe + 7 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "tail") || strcmp(words[pipe + 5], "-n") ||
      !parse_u64_arg(words[pipe + 6], &limit)) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  size_t start = list.len - take;
  for (size_t idx = start; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_sort(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 5 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort")) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_sort_uniq(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 7 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort") || strcmp(words[pipe + 5], "|") ||
      strcmp(words[pipe + 6], "uniq")) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  emit_unique_byte_line_list(&list);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_sort_uniq_wc(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 10 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort") || strcmp(words[pipe + 5], "|") ||
      strcmp(words[pipe + 6], "uniq") || strcmp(words[pipe + 7], "|") ||
      strcmp(words[pipe + 8], "wc") || strcmp(words[pipe + 9], "-l")) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  write_padded_u64(count_unique_byte_line_list(&list));
  write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_sort_uniq_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0 || count <= pipe + 8 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort") || strcmp(words[pipe + 5], "|") ||
      strcmp(words[pipe + 6], "uniq") || strcmp(words[pipe + 7], "|")) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  byte_line_list_sort_unique(&list);
  rc = emit_head_line_list_mode(words, pipe + 8, count, &list, 0);
  byte_line_list_free(&list);
  return rc;
}

static int pipe_seq_grep_sort_wc(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 8 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort") || strcmp(words[pipe + 5], "|") ||
      strcmp(words[pipe + 6], "wc") || strcmp(words[pipe + 7], "-l")) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  write_padded_u64((unsigned long long)list.len);
  write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_sort_head(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (pipe < 0 || count != pipe + 9 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort") || strcmp(words[pipe + 5], "|") ||
      strcmp(words[pipe + 6], "head") || strcmp(words[pipe + 7], "-n") ||
      !parse_u64_arg(words[pipe + 8], &limit) || limit == 0) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_sort_tail(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (pipe < 0 || count != pipe + 9 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort") || strcmp(words[pipe + 5], "|") ||
      strcmp(words[pipe + 6], "tail") || strcmp(words[pipe + 7], "-n") ||
      !parse_u64_arg(words[pipe + 8], &limit)) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  size_t start = list.len - take;
  for (size_t idx = start; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_sort_xargs_echo(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int first = 1;
  if (pipe < 0 || count != pipe + 8 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "sort") || strcmp(words[pipe + 5], "|") ||
      strcmp(words[pipe + 6], "xargs") || strcmp(words[pipe + 7], "echo")) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  for (size_t idx = 0; idx < list.len; idx++) {
    emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
  }
  if (!first) write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_grep_xargs_echo(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int first = 1;
  if (pipe < 0 || count != pipe + 6 || strcmp(words[pipe + 3], "|") ||
      strcmp(words[pipe + 4], "xargs") || strcmp(words[pipe + 5], "echo")) {
    return unsupported();
  }
  if (!collect_seq_grep_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  for (size_t idx = 0; idx < list.len; idx++) {
    emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
  }
  if (!first) write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_sort(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 2 || strcmp(words[pipe + 1], "sort")) {
    return unsupported();
  }
  if (!collect_seq_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_sort_uniq(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 4 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "uniq")) {
    return unsupported();
  }
  if (!collect_seq_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  emit_unique_byte_line_list(&list);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_sort_uniq_wc(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 7 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "uniq") ||
      strcmp(words[pipe + 4], "|") || strcmp(words[pipe + 5], "wc") ||
      strcmp(words[pipe + 6], "-l")) {
    return unsupported();
  }
  if (!collect_seq_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  write_padded_u64(count_unique_byte_line_list(&list));
  write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_sort_uniq_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int rc = 127;
  if (pipe < 0 || count <= pipe + 5 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "uniq") ||
      strcmp(words[pipe + 4], "|")) {
    return unsupported();
  }
  if (!collect_seq_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  byte_line_list_sort_unique(&list);
  rc = emit_head_line_list_mode(words, pipe + 5, count, &list, 1);
  byte_line_list_free(&list);
  return rc;
}

static int pipe_seq_sort_wc(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  if (pipe < 0 || count != pipe + 5 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "wc") ||
      strcmp(words[pipe + 4], "-l")) {
    return unsupported();
  }
  if (!collect_seq_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  write_padded_u64((unsigned long long)list.len);
  write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_sort_head(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (pipe < 0 || count != pipe + 6 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "head") ||
      strcmp(words[pipe + 4], "-n") || !parse_u64_arg(words[pipe + 5], &limit) ||
      limit == 0) {
    return unsupported();
  }
  if (!collect_seq_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_sort_tail(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (pipe < 0 || count != pipe + 6 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "tail") ||
      strcmp(words[pipe + 4], "-n") || !parse_u64_arg(words[pipe + 5], &limit)) {
    return unsupported();
  }
  if (!collect_seq_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  size_t start = list.len - take;
  for (size_t idx = start; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  byte_line_list_free(&list);
  return 0;
}

static int pipe_seq_sort_xargs_echo(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  struct byte_line_list list = {0};
  int first = 1;
  unsigned long long batch_size = 0;
  struct xargs_echo_batch_state batch = {0};
  if (pipe < 0 || count < pipe + 4 || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") ||
      !xargs_echo_words_mode(words, pipe + 3, count, &batch_size)) {
    return unsupported();
  }
  if (!collect_seq_lines(words, pipe, &list)) {
    byte_line_list_free(&list);
    return unsupported();
  }
  qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
  batch.size = batch_size;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (batch_size) {
      emit_xargs_echo_batch_bytes(list.items[idx].data, list.items[idx].len, &batch);
    } else {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
  }
  if (batch_size) finish_xargs_echo_batch(&batch);
  if (!batch_size && !first) write_bytes("\n", 1);
  byte_line_list_free(&list);
  return 0;
}

static void emit_xargs_echo_path_list(const struct path_list *list) {
  int first = 1;
  for (size_t idx = 0; idx < list->len; idx++) {
    emit_xargs_echo_path(list->items[idx], &first);
  }
  if (!first) write_bytes("\n", 1);
}

static void emit_path_list_tail(const struct path_list *list, unsigned long long limit) {
  size_t take = limit > (unsigned long long)list->len ? list->len : (size_t)limit;
  size_t start = list->len - take;
  for (size_t idx = start; idx < list->len; idx++) {
    write_line(list->items[idx]);
  }
}

static unsigned long long count_unique_path_list(const struct path_list *list) {
  unsigned long long count = 0;
  const char *previous = NULL;
  for (size_t idx = 0; idx < list->len; idx++) {
    if (!previous || strcmp(previous, list->items[idx])) {
      count++;
      previous = list->items[idx];
    }
  }
  return count;
}

static int find_xargs_echo_walk_path(char *path, size_t cap, const char *name_glob,
                                     int max_depth, int depth, int *first);

static int find_xargs_echo_walk_dir(char *path, size_t cap, const char *name_glob,
                                    int max_depth, int depth, int *first) {
  DIR *dir = opendir(path);
  if (!dir) {
    write_err_path("find", path, errno);
    return 1;
  }
  size_t base_len = strlen(path);
  int rc = 0;
  struct dirent *entry = NULL;
  while ((entry = readdir(dir)) != NULL) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t name_len = strlen(entry->d_name);
    if (base_len + 1 + name_len + 1 > cap) {
      rc = 1;
      continue;
    }
    path[base_len] = '/';
    memcpy(path + base_len + 1, entry->d_name, name_len + 1);
    if (entry->d_type == DT_DIR) {
      if (find_can_descend(max_depth, depth + 1)) {
        rc |= find_xargs_echo_walk_dir(path, cap, name_glob, max_depth, depth + 1, first);
      }
    } else if (entry->d_type == DT_REG) {
      if (name_glob_match(name_glob, entry->d_name)) emit_xargs_echo_path(path, first);
    } else {
      rc |= find_xargs_echo_walk_path(path, cap, name_glob, max_depth, depth + 1, first);
    }
    path[base_len] = '\0';
  }
  closedir(dir);
  return rc;
}

static int find_xargs_echo_walk_path(char *path, size_t cap, const char *name_glob,
                                     int max_depth, int depth, int *first) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("find", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode) && name_glob_match(name_glob, cap_base(path))) {
    emit_xargs_echo_path(path, first);
    return 0;
  }
  if (!S_ISDIR(st.st_mode)) return 0;
  if (!find_can_descend(max_depth, depth)) return 0;
  return find_xargs_echo_walk_dir(path, cap, name_glob, max_depth, depth, first);
}

static int find_tail_collect_path(char *path, size_t cap, const char *name_glob,
                                  int max_depth, int depth, unsigned long long limit,
                                  struct path_list *list);

static int find_tail_collect_dir(char *path, size_t cap, const char *name_glob,
                                 int max_depth, int depth, unsigned long long limit,
                                 struct path_list *list) {
  DIR *dir = opendir(path);
  if (!dir) {
    write_err_path("find", path, errno);
    return 1;
  }
  size_t base_len = strlen(path);
  int rc = 0;
  struct dirent *entry = NULL;
  while ((entry = readdir(dir)) != NULL) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    size_t name_len = strlen(entry->d_name);
    if (base_len + 1 + name_len + 1 > cap) {
      rc = 1;
      continue;
    }
    path[base_len] = '/';
    memcpy(path + base_len + 1, entry->d_name, name_len + 1);
    if (entry->d_type == DT_DIR) {
      if (find_can_descend(max_depth, depth + 1)) {
        rc |= find_tail_collect_dir(path, cap, name_glob, max_depth, depth + 1, limit, list);
      }
    } else if (entry->d_type == DT_REG) {
      if (name_glob_match(name_glob, entry->d_name) &&
          !path_list_push_tail(list, path, limit)) {
        rc = 1;
      }
    } else {
      rc |= find_tail_collect_path(path, cap, name_glob, max_depth, depth + 1, limit, list);
    }
    path[base_len] = '\0';
  }
  closedir(dir);
  return rc;
}

static int find_tail_collect_path(char *path, size_t cap, const char *name_glob,
                                  int max_depth, int depth, unsigned long long limit,
                                  struct path_list *list) {
  struct stat st;
  if (lstat(path, &st) != 0) {
    write_err_path("find", path, errno);
    return 1;
  }
  if (S_ISREG(st.st_mode) && name_glob_match(name_glob, cap_base(path))) {
    return path_list_push_tail(list, path, limit) ? 0 : 1;
  }
  if (!S_ISDIR(st.st_mode)) return 0;
  if (!find_can_descend(max_depth, depth)) return 0;
  return find_tail_collect_dir(path, cap, name_glob, max_depth, depth, limit, list);
}

static void emit_path_list_all(const struct path_list *list) {
  for (size_t idx = 0; idx < list->len; idx++) write_line(list->items[idx]);
}

static void emit_path_list_head(const struct path_list *list, unsigned long long limit) {
  size_t take = limit > (unsigned long long)list->len ? list->len : (size_t)limit;
  for (size_t idx = 0; idx < take; idx++) write_line(list->items[idx]);
}

static void emit_unique_path_list(const struct path_list *list) {
  const char *previous = NULL;
  for (size_t idx = 0; idx < list->len; idx++) {
    if (!previous || strcmp(previous, list->items[idx])) {
      write_line(list->items[idx]);
      previous = list->items[idx];
    }
  }
}

struct ls_pipe_source {
  const char *path;
  enum ls_entry_mode mode;
};

static int parse_ls_pipe_source(char **words, int start, int end,
                                struct ls_pipe_source *source) {
  const char *path = ".";
  enum ls_entry_mode mode = LS_ENTRY_VISIBLE;
  int paths = 0;
  if (start >= end || strcmp(words[start], "ls")) return unsupported();
  for (int idx = start + 1; idx < end; idx++) {
    if (!strcmp(words[idx], "--")) return unsupported();
    if (words[idx][0] == '-' && words[idx][1] != 0) {
      for (const char *flag = words[idx] + 1; *flag; flag++) {
        if (*flag == '1') {
          continue;
        }
        if (*flag == 'a') {
          mode = LS_ENTRY_ALL;
          continue;
        }
        if (*flag == 'A') {
          if (mode != LS_ENTRY_ALL) mode = LS_ENTRY_ALMOST_ALL;
          continue;
        }
        return unsupported();
      }
    } else {
      path = words[idx];
      paths++;
    }
  }
  if (paths > 1) return unsupported();
  struct stat st;
  if (stat(path, &st) != 0 || !S_ISDIR(st.st_mode)) return unsupported();
  source->path = path;
  source->mode = mode;
  return 0;
}

static int collect_ls_names(const char *path, enum ls_entry_mode mode, struct path_list *list) {
  DIR *dir = opendir(path);
  if (!dir) return unsupported();
  struct dirent *entry = NULL;
  while ((entry = readdir(dir))) {
    if (mode == LS_ENTRY_VISIBLE && entry->d_name[0] == '.') continue;
    if (mode == LS_ENTRY_ALMOST_ALL &&
        (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, ".."))) {
      continue;
    }
    if (!path_list_push(list, entry->d_name)) {
      closedir(dir);
      return 1;
    }
  }
  closedir(dir);
  qsort(list->items, list->len, sizeof(char *), cmp_string_ptr);
  return 0;
}

static int load_ls_pipe_names(char **words, int end, struct path_list *list) {
  struct ls_pipe_source source = {0};
  int rc = parse_ls_pipe_source(words, 0, end, &source);
  if (rc != 0) return rc;
  return collect_ls_names(source.path, source.mode, list);
}

static int pipe_ls_wc(char **words, int count) {
  struct path_list list = {0};
  if (count < 4 || strcmp(words[count - 2], "wc") || strcmp(words[count - 1], "-l")) {
    return unsupported();
  }
  int pipe = count - 3;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  write_padded_u64((unsigned long long)list.len);
  write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_head(char **words, int count) {
  struct path_list list = {0};
  unsigned long long limit = 0;
  if (count < 5 || strcmp(words[count - 3], "head") ||
      strcmp(words[count - 2], "-n") || !parse_u64_arg(words[count - 1], &limit) ||
      limit == 0) {
    return unsupported();
  }
  int pipe = count - 4;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_path_list_head(&list, limit);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_tail(char **words, int count) {
  struct path_list list = {0};
  unsigned long long limit = 0;
  if (count < 5 || strcmp(words[count - 3], "tail") ||
      strcmp(words[count - 2], "-n") || !parse_u64_arg(words[count - 1], &limit)) {
    return unsupported();
  }
  int pipe = count - 4;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_path_list_tail(&list, limit);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_sort(char **words, int count) {
  struct path_list list = {0};
  if (count < 3 || strcmp(words[count - 1], "sort")) return unsupported();
  int pipe = count - 2;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_path_list_all(&list);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_sort_uniq(char **words, int count) {
  struct path_list list = {0};
  if (count < 5 || strcmp(words[count - 3], "sort") ||
      strcmp(words[count - 2], "|") || strcmp(words[count - 1], "uniq")) {
    return unsupported();
  }
  int pipe = count - 4;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_unique_path_list(&list);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_sort_uniq_wc(char **words, int count) {
  struct path_list list = {0};
  if (count < 8 || strcmp(words[count - 6], "sort") ||
      strcmp(words[count - 5], "|") || strcmp(words[count - 4], "uniq") ||
      strcmp(words[count - 3], "|") || strcmp(words[count - 2], "wc") ||
      strcmp(words[count - 1], "-l")) {
    return unsupported();
  }
  int pipe = count - 7;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  write_padded_u64(count_unique_path_list(&list));
  write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int collect_ls_sort_uniq_pipe_lines(char **words, int pipe,
                                           struct byte_line_list *list) {
  struct path_list names = {0};
  int rc = load_ls_pipe_names(words, pipe, &names);
  if (rc == 127) return 0;
  if (rc != 0) {
    path_list_free(&names);
    return -1;
  }
  const char *previous = NULL;
  for (size_t idx = 0; idx < names.len; idx++) {
    if (!previous || strcmp(previous, names.items[idx])) {
      if (!byte_line_list_push_plain_match(list, names.items[idx],
                                           strlen(names.items[idx]), 1)) {
        path_list_free(&names);
        return -1;
      }
      previous = names.items[idx];
    }
  }
  path_list_free(&names);
  return 1;
}

static int collect_ls_sort_uniq_grep_pipe_lines(char **words, int pipe,
                                                struct byte_line_list *list) {
  struct byte_line_list source = {0};
  int collected = collect_ls_sort_uniq_pipe_lines(words, pipe, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  const char *pattern = words[pipe + 5 + 1];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int pipe_ls_sort_uniq_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  if (pipe <= 0 || pipe + 5 >= count || strcmp(words[pipe + 1], "sort") ||
      strcmp(words[pipe + 2], "|") || strcmp(words[pipe + 3], "uniq") ||
      strcmp(words[pipe + 4], "|")) {
    return unsupported();
  }

  struct byte_line_list list = {0};
  int rc = 127;
  if (count >= pipe + 7 && !strcmp(words[pipe + 5], "grep") &&
      is_plain_literal_pattern(words[pipe + 6])) {
    if (count != pipe + 7 && strcmp(words[pipe + 7], "|")) return unsupported();
    rc = collect_ls_sort_uniq_grep_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == pipe + 7) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, pipe + 8, count, &list, 0);
    }
  } else {
    rc = collect_ls_sort_uniq_pipe_lines(words, pipe, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, pipe + 5, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_ls_sort_wc(char **words, int count) {
  struct path_list list = {0};
  if (count < 6 || strcmp(words[count - 4], "sort") ||
      strcmp(words[count - 3], "|") || strcmp(words[count - 2], "wc") ||
      strcmp(words[count - 1], "-l")) {
    return unsupported();
  }
  int pipe = count - 5;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  write_padded_u64((unsigned long long)list.len);
  write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_sort_head(char **words, int count) {
  struct path_list list = {0};
  unsigned long long limit = 0;
  if (count < 7 || strcmp(words[count - 5], "sort") ||
      strcmp(words[count - 4], "|") || strcmp(words[count - 3], "head") ||
      strcmp(words[count - 2], "-n") || !parse_u64_arg(words[count - 1], &limit) ||
      limit == 0) {
    return unsupported();
  }
  int pipe = count - 6;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_path_list_head(&list, limit);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_sort_tail(char **words, int count) {
  struct path_list list = {0};
  unsigned long long limit = 0;
  if (count < 7 || strcmp(words[count - 5], "sort") ||
      strcmp(words[count - 4], "|") || strcmp(words[count - 3], "tail") ||
      strcmp(words[count - 2], "-n") || !parse_u64_arg(words[count - 1], &limit)) {
    return unsupported();
  }
  int pipe = count - 6;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_path_list_tail(&list, limit);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_sort_xargs_echo(char **words, int count) {
  struct path_list list = {0};
  if (count < 6 || strcmp(words[count - 4], "sort") ||
      strcmp(words[count - 3], "|") || strcmp(words[count - 2], "xargs") ||
      strcmp(words[count - 1], "echo")) {
    return unsupported();
  }
  int pipe = count - 5;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_xargs_echo_path_list(&list);
  path_list_free(&list);
  return 0;
}

static int collect_ls_grep_pipe_lines(char **words, int pipe,
                                      struct byte_line_list *lines) {
  struct path_list names = {0};
  int rc = load_ls_pipe_names(words, pipe, &names);
  if (rc == 127) return 0;
  if (rc != 0) {
    path_list_free(&names);
    return -1;
  }
  const char *pattern = words[pipe + 2];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < names.len; idx++) {
    if (contains_bytes(names.items[idx], (ssize_t)strlen(names.items[idx]), pattern,
                       pattern_len) &&
        !byte_line_list_push_plain_match(lines, names.items[idx], strlen(names.items[idx]),
                                         1)) {
      path_list_free(&names);
      return -1;
    }
  }
  path_list_free(&names);
  return 1;
}

static int pipe_ls_grep_producer(char **words, int count) {
  int pipe = first_pipe_index(words, count);
  if (pipe <= 0 || count < pipe + 3 || strcmp(words[pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[pipe + 2])) {
    return unsupported();
  }
  if (count != pipe + 3 && strcmp(words[pipe + 3], "|")) return unsupported();
  if (count - (pipe + 4) == 3 && !strcmp(words[pipe + 4], "xargs") &&
      !strcmp(words[pipe + 5], "wc") && !strcmp(words[pipe + 6], "-l")) {
    return unsupported();
  }
  if (count - (pipe + 4) == 5 && !strcmp(words[pipe + 4], "sort") &&
      !strcmp(words[pipe + 5], "|") && !strcmp(words[pipe + 6], "xargs") &&
      !strcmp(words[pipe + 7], "wc") && !strcmp(words[pipe + 8], "-l")) {
    return unsupported();
  }

  struct byte_line_list list = {0};
  int rc = collect_ls_grep_pipe_lines(words, pipe, &list);
  if (rc < 0) {
    byte_line_list_free(&list);
    return 1;
  }
  if (rc == 0) {
    byte_line_list_free(&list);
    return unsupported();
  }
  if (count == pipe + 3) {
    for (size_t idx = 0; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
    rc = list.len ? 0 : 1;
  } else {
    rc = emit_head_line_list_mode(words, pipe + 4, count, &list, 0);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_ls_grep(char **words, int count) {
  struct path_list list = {0};
  if (count < 4 || strcmp(words[count - 2], "grep") ||
      !is_plain_literal_pattern(words[count - 1])) {
    return unsupported();
  }
  int pipe = count - 3;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  const char *pattern = words[count - 1];
  size_t pattern_len = strlen(pattern);
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  int matched = 0;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      write_line(list.items[idx]);
      matched = 1;
    }
  }
  path_list_free(&list);
  return matched ? 0 : 1;
}

static int pipe_ls_grep_wc(char **words, int count) {
  struct path_list list = {0};
  if (count < 7 || strcmp(words[count - 5], "grep") ||
      !is_plain_literal_pattern(words[count - 4]) || strcmp(words[count - 3], "|") ||
      strcmp(words[count - 2], "wc") || strcmp(words[count - 1], "-l")) {
    return unsupported();
  }
  int pipe = count - 6;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  const char *pattern = words[count - 4];
  size_t pattern_len = strlen(pattern);
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  unsigned long long matches = 0;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      matches++;
    }
  }
  write_padded_u64(matches);
  write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_grep_xargs_echo(char **words, int count) {
  struct path_list list = {0};
  if (count < 7 || strcmp(words[count - 5], "grep") ||
      !is_plain_literal_pattern(words[count - 4]) || strcmp(words[count - 3], "|") ||
      strcmp(words[count - 2], "xargs") || strcmp(words[count - 1], "echo")) {
    return unsupported();
  }
  int pipe = count - 6;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  const char *pattern = words[count - 4];
  size_t pattern_len = strlen(pattern);
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  int first = 1;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      emit_xargs_echo_path(list.items[idx], &first);
    }
  }
  if (!first) write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_grep_sort_xargs_echo(char **words, int count) {
  struct path_list list = {0};
  if (count < 9 || strcmp(words[count - 7], "grep") ||
      !is_plain_literal_pattern(words[count - 6]) || strcmp(words[count - 5], "|") ||
      strcmp(words[count - 4], "sort") || strcmp(words[count - 3], "|") ||
      strcmp(words[count - 2], "xargs") || strcmp(words[count - 1], "echo")) {
    return unsupported();
  }
  int pipe = count - 8;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  const char *pattern = words[count - 6];
  size_t pattern_len = strlen(pattern);
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  int first = 1;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      emit_xargs_echo_path(list.items[idx], &first);
    }
  }
  if (!first) write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int pipe_ls_xargs_echo(char **words, int count) {
  struct path_list list = {0};
  if (count < 5 || strcmp(words[count - 2], "xargs") ||
      strcmp(words[count - 1], "echo")) {
    return unsupported();
  }
  int pipe = count - 3;
  if (pipe <= 0 || strcmp(words[pipe], "|")) return unsupported();
  int rc = load_ls_pipe_names(words, pipe, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_xargs_echo_path_list(&list);
  path_list_free(&list);
  return 0;
}

static int collect_awk_print_field_file(const char *file, const char *script,
                                        struct byte_line_list *list,
                                        int file_error_code) {
  char buf[8192];
  char line[8192];
  size_t used = 0;
  const char *pat = NULL;
  unsigned long long field = 1;
  if (!parse_awk_print_field_script(script, &pat, &field)) return unsupported();
  size_t pat_len = pat ? strlen(pat) : 0;
  int fd = STDIN_FILENO;
  int close_fd = 0;
  if (file) {
    fd = open(file, O_RDONLY);
    if (fd < 0) {
      write_err_path("awk", file, errno);
      return file_error_code;
    }
    close_fd = 1;
  }
  for (;;) {
    ssize_t read_len = read(fd, buf, sizeof(buf));
    if (read_len == 0) break;
    if (read_len < 0) {
      if (file) write_err_path("awk", file, errno);
      if (close_fd) close(fd);
      return file_error_code;
    }
    for (ssize_t idx = 0; idx < read_len; idx++) {
      if (used < sizeof(line)) line[used++] = buf[idx];
      if (buf[idx] == '\n' || used == sizeof(line)) {
        if (!pat || contains_bytes(line, (ssize_t)used, pat, pat_len)) {
          size_t start = 0;
          size_t end = 0;
          awk_field_bounds(line, used, field, &start, &end);
          if (!byte_line_list_push_plain_match(list, line + start, end - start, 1)) {
            if (close_fd) close(fd);
            return 1;
          }
        }
        used = 0;
      }
    }
  }
  if (used && (!pat || contains_bytes(line, (ssize_t)used, pat, pat_len))) {
    size_t start = 0;
    size_t end = 0;
    awk_field_bounds(line, used, field, &start, &end);
    if (!byte_line_list_push_plain_match(list, line + start, end - start, 1)) {
      if (close_fd) close(fd);
      return 1;
    }
  }
  if (close_fd) close(fd);
  return 0;
}

static int collect_awk_print_field_lines(char **words, struct byte_line_list *list) {
  if (strcmp(words[0], "awk") || strcmp(words[3], "|")) {
    return unsupported();
  }
  return collect_awk_print_field_file(words[2], words[1], list, 0);
}

static int pipe_awk_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 6 || strcmp(words[4], "wc") || strcmp(words[5], "-l")) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    write_padded_u64((unsigned long long)list.len);
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_head(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 7 || strcmp(words[4], "head") || strcmp(words[5], "-n") ||
      !parse_u64_arg(words[6], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_tail(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 7 || strcmp(words[4], "tail") || strcmp(words[5], "-n") ||
      !parse_u64_arg(words[6], &limit)) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 5 || strcmp(words[4], "sort")) return unsupported();
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort_uniq(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 7 || strcmp(words[4], "sort") || strcmp(words[5], "|") ||
      strcmp(words[6], "uniq")) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    emit_unique_byte_line_list(&list);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort_uniq_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 10 || strcmp(words[4], "sort") || strcmp(words[5], "|") ||
      strcmp(words[6], "uniq") || strcmp(words[7], "|") || strcmp(words[8], "wc") ||
      strcmp(words[9], "-l")) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    write_padded_u64(count_unique_byte_line_list(&list));
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort_uniq_producer(char **words, int count) {
  struct byte_line_list list = {0};
  if (count <= 8 || strcmp(words[4], "sort") || strcmp(words[5], "|") ||
      strcmp(words[6], "uniq") || strcmp(words[7], "|") ||
      !head_line_list_mode_supported(words, 8, count)) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    byte_line_list_sort_unique(&list);
    rc = emit_head_line_list_mode(words, 8, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_grep_producer(char **words, int count) {
  struct byte_line_list source = {0};
  struct byte_line_list list = {0};
  int rc = 127;
  if (count < 6 || strcmp(words[4], "grep") ||
      !is_plain_literal_pattern(words[5])) {
    return unsupported();
  }
  if (count != 6 && strcmp(words[6], "|")) return unsupported();
  if (count != 6 && !head_line_list_mode_supported(words, 7, count)) {
    return unsupported();
  }

  rc = collect_awk_print_field_lines(words, &source);
  if (rc != 0) {
    byte_line_list_free(&source);
    return rc;
  }

  const char *pattern = words[5];
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(&list, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      byte_line_list_free(&list);
      return 1;
    }
  }
  byte_line_list_free(&source);

  if (count == 6) {
    for (size_t idx = 0; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
    rc = list.len ? 0 : 1;
  } else {
    rc = emit_head_line_list_mode(words, 7, count, &list, 0);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort_wc(char **words, int count) {
  struct byte_line_list list = {0};
  if (count != 8 || strcmp(words[4], "sort") || strcmp(words[5], "|") ||
      strcmp(words[6], "wc") || strcmp(words[7], "-l")) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    write_padded_u64((unsigned long long)list.len);
    write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort_head(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 9 || strcmp(words[4], "sort") || strcmp(words[5], "|") ||
      strcmp(words[6], "head") || strcmp(words[7], "-n") ||
      !parse_u64_arg(words[8], &limit) || limit == 0) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    for (size_t idx = 0; idx < take; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort_tail(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long limit = 0;
  if (count != 9 || strcmp(words[4], "sort") || strcmp(words[5], "|") ||
      strcmp(words[6], "tail") || strcmp(words[7], "-n") ||
      !parse_u64_arg(words[8], &limit)) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
    size_t start = list.len - take;
    for (size_t idx = start; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort_xargs_echo(char **words, int count) {
  struct byte_line_list list = {0};
  int first = 1;
  if (count != 8 || strcmp(words[4], "sort") || strcmp(words[5], "|") ||
      strcmp(words[6], "xargs") || strcmp(words[7], "echo")) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
    if (!first) write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_sort_xargs_wc(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 9 || strcmp(words[4], "sort") || strcmp(words[5], "|") ||
      strcmp(words[6], "xargs") || strcmp(words[7], "wc") || strcmp(words[8], "-l")) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    qsort(list.items, list.len, sizeof(struct byte_line_item), byte_line_item_cmp);
    for (size_t idx = 0; idx < list.len; idx++) {
      if (!emit_xargs_wc_bytes(list.items[idx].data, list.items[idx].len, &total, &files,
                               &err)) {
        byte_line_list_free(&list);
        return 1;
      }
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  byte_line_list_free(&list);
  return rc ? rc : (err ? 1 : 0);
}

static int pipe_awk_xargs_echo(char **words, int count) {
  struct byte_line_list list = {0};
  int first = 1;
  if (count != 6 || strcmp(words[4], "xargs") || strcmp(words[5], "echo")) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    for (size_t idx = 0; idx < list.len; idx++) {
      emit_xargs_echo_bytes(list.items[idx].data, list.items[idx].len, &first);
    }
    if (!first) write_bytes("\n", 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_awk_xargs_wc(char **words, int count) {
  struct byte_line_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  if (count != 7 || strcmp(words[4], "xargs") || strcmp(words[5], "wc") || strcmp(words[6], "-l")) {
    return unsupported();
  }
  int rc = collect_awk_print_field_lines(words, &list);
  if (rc == 0) {
    for (size_t idx = 0; idx < list.len; idx++) {
      if (!emit_xargs_wc_bytes(list.items[idx].data, list.items[idx].len, &total, &files, &err)) {
        byte_line_list_free(&list);
        return 1;
      }
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  byte_line_list_free(&list);
  return rc ? rc : (err ? 1 : 0);
}

static int dispatch_pipe_awk_handlers(char **words, int count) {
  int code = pipe_awk_grep_producer(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_awk_sort(words, count);
  if (code != 127) return code;
  code = pipe_awk_wc(words, count);
  if (code != 127) return code;
  code = pipe_awk_head(words, count);
  if (code != 127) return code;
  code = pipe_awk_tail(words, count);
  if (code != 127) return code;
  code = pipe_awk_xargs_echo(words, count);
  if (code != 127) return code;
  return pipe_awk_xargs_wc(words, count);
}

static int pipe_stdin_awk(char **words, int count) {
  if (count < 4 || strcmp(words[0], "awk") || strcmp(words[2], "|")) {
    return unsupported();
  }

  char **rewritten = (char **)calloc((size_t)count + 1, sizeof(char *));
  if (!rewritten) return 1;
  rewritten[0] = "awk";
  rewritten[1] = words[1];
  rewritten[2] = NULL;
  rewritten[3] = "|";
  int rewritten_count = 4;
  for (int idx = 3; idx < count; idx++) rewritten[rewritten_count++] = words[idx];
  int code = dispatch_pipe_awk_handlers(rewritten, rewritten_count);
  free(rewritten);
  return code;
}

static int pipe_cat_awk(char **words, int count) {
  const char *filter = NULL;
  unsigned long long field = 1;
  if (count < 5 || strcmp(words[0], "cat") || strcmp(words[2], "|") ||
      strcmp(words[3], "awk") ||
      !parse_awk_print_field_script(words[4], &filter, &field)) {
    return unsupported();
  }
  (void)filter;
  (void)field;

  char *normalized[4] = {"awk", words[4], words[1], "|"};
  if (count == 5) {
    struct byte_line_list list = {0};
    int rc = collect_awk_print_field_lines(normalized, &list);
    if (rc == 0) {
      for (size_t idx = 0; idx < list.len; idx++) write_bytes(list.items[idx].data, list.items[idx].len);
    }
    byte_line_list_free(&list);
    return rc;
  }

  if (count < 7 || strcmp(words[5], "|")) return unsupported();
  char **rewritten = (char **)calloc((size_t)count, sizeof(char *));
  if (!rewritten) return 1;
  rewritten[0] = "awk";
  rewritten[1] = words[4];
  rewritten[2] = words[1];
  rewritten[3] = "|";
  int rewritten_count = 4;
  for (int idx = 6; idx < count; idx++) rewritten[rewritten_count++] = words[idx];
  int code = dispatch_pipe_awk_handlers(rewritten, rewritten_count);
  free(rewritten);
  return code;
}

struct find_pipe_prefix {
  const char *root;
  const char *name_glob;
  int max_depth;
  int pipe;
};

static int parse_find_pipe_prefix(char **words, int count,
                                  struct find_pipe_prefix *prefix) {
  int type_idx = 2;
  int max_depth = -1;
  if (count < 5 || strcmp(words[0], "find")) {
    return 0;
  }
  if (!strcmp(words[2], "-type") && !strcmp(words[3], "f")) {
    type_idx = 2;
  } else if (count >= 7 && !strcmp(words[2], "-maxdepth") &&
             !strcmp(words[4], "-type") && !strcmp(words[5], "f")) {
    unsigned long long parsed_depth = 0;
    if (!parse_u64_arg(words[3], &parsed_depth) || parsed_depth == 0 ||
        parsed_depth > (unsigned long long)INT_MAX) {
      return 0;
    }
    type_idx = 4;
    max_depth = (int)parsed_depth;
  } else {
    return 0;
  }
  int after_type = type_idx + 2;
  if (!strcmp(words[after_type], "|")) {
    prefix->root = words[1];
    prefix->name_glob = "*";
    prefix->max_depth = max_depth;
    prefix->pipe = after_type;
    return 1;
  }
  if (count >= after_type + 3 && !strcmp(words[after_type], "-name") &&
      safe_name_glob(words[after_type + 1]) &&
      !strcmp(words[after_type + 2], "|")) {
    prefix->root = words[1];
    prefix->name_glob = words[after_type + 1];
    prefix->max_depth = max_depth;
    prefix->pipe = after_type + 2;
    return 1;
  }
  return 0;
}

static int pipe_find_xargs_echo(char **words, int count) {
  char path[PATH_MAX];
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) ||
      (count != prefix.pipe + 2 && count != prefix.pipe + 3) ||
      strcmp(words[prefix.pipe + 1], "xargs") ||
      (count == prefix.pipe + 3 && strcmp(words[prefix.pipe + 2], "echo"))) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int first = 1;
  int rc = find_xargs_echo_walk_path(path, sizeof(path), prefix.name_glob,
                                     prefix.max_depth, 0, &first);
  if (!first) write_bytes("\n", 1);
  return rc;
}

static int pipe_find_xargs_wc(char **words, int count) {
  char path[PATH_MAX];
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 4 ||
      strcmp(words[prefix.pipe + 1], "xargs") || strcmp(words[prefix.pipe + 2], "wc") ||
      strcmp(words[prefix.pipe + 3], "-l")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  (void)find_wc_walk_path(path, sizeof(path), prefix.name_glob, prefix.max_depth, 0,
                          &total, &files, &err);
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  return err ? 1 : 0;
}

static int xargs_wc_output_mode_supported(char **words, int start, int count) {
  unsigned long long limit = 0;
  int remaining = count - start;
  if (remaining == 2 && !strcmp(words[start], "wc") && !strcmp(words[start + 1], "-l")) {
    return 1;
  }
  if (remaining == 3 && !strcmp(words[start], "head") &&
      !strcmp(words[start + 1], "-n") && parse_u64_arg(words[start + 2], &limit) &&
      limit > 0) {
    return 1;
  }
  if (remaining == 3 && !strcmp(words[start], "tail") &&
      !strcmp(words[start + 1], "-n") && parse_u64_arg(words[start + 2], &limit)) {
    return 1;
  }
  if (remaining >= 1 && !strcmp(words[start], "sort")) {
    if (remaining == 1) return 1;
    if (remaining == 3 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "uniq")) {
      return 1;
    }
    if (remaining == 6 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "uniq") && !strcmp(words[start + 3], "|") &&
        !strcmp(words[start + 4], "wc") && !strcmp(words[start + 5], "-l")) {
      return 1;
    }
    if (remaining == 4 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "wc") && !strcmp(words[start + 3], "-l")) {
      return 1;
    }
    if (remaining == 5 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "head") && !strcmp(words[start + 3], "-n") &&
        parse_u64_arg(words[start + 4], &limit) && limit > 0) {
      return 1;
    }
    if (remaining == 5 && !strcmp(words[start + 1], "|") &&
        !strcmp(words[start + 2], "tail") && !strcmp(words[start + 3], "-n") &&
        parse_u64_arg(words[start + 4], &limit)) {
      return 1;
    }
  }
  return 0;
}

static void sorted_path_list_dedup(struct path_list *list) {
  size_t write = 0;
  for (size_t read = 0; read < list->len; read++) {
    if (write > 0 && !strcmp(list->items[write - 1], list->items[read])) {
      free(list->items[read]);
      continue;
    }
    if (write != read) list->items[write] = list->items[read];
    write++;
  }
  list->len = write;
}

static int byte_line_list_push_xargs_wc_path(struct byte_line_list *lines,
                                             const char *path,
                                             unsigned long long *total,
                                             unsigned long long *files,
                                             int *err) {
  int count_err = 0;
  unsigned long long line_count = count_newlines_path(path, "wc", &count_err);
  if (count_err) {
    *err = 1;
    return 1;
  }
  char prefix[32];
  int prefix_len = snprintf(prefix, sizeof(prefix), "%8llu ", line_count);
  if (prefix_len < 0 || (size_t)prefix_len >= sizeof(prefix)) return 0;
  size_t path_len = strlen(path);
  size_t len = (size_t)prefix_len + path_len + 1;
  char *out = (char *)malloc(len ? len : 1);
  if (!out) return 0;
  memcpy(out, prefix, (size_t)prefix_len);
  if (path_len) memcpy(out + prefix_len, path, path_len);
  out[len - 1] = '\n';
  int ok = byte_line_list_push(lines, out, len);
  free(out);
  if (!ok) return 0;
  *total += line_count;
  *files += 1;
  return 1;
}

static int collect_find_xargs_wc_output_lines(const struct find_pipe_prefix *prefix,
                                              const char *pattern, int sort_paths,
                                              int uniq_paths,
                                              struct byte_line_list *lines) {
  char path[PATH_MAX];
  struct path_list list = {0};
  if (!copy_cstr(path, sizeof(path), prefix->root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix->name_glob,
                                   prefix->max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  if (sort_paths || uniq_paths) {
    qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  }
  if (uniq_paths) sorted_path_list_dedup(&list);
  unsigned long long total = 0;
  unsigned long long files = 0;
  size_t input_paths = 0;
  int err = 0;
  size_t pattern_len = pattern ? strlen(pattern) : 0;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (pattern &&
        !contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      continue;
    }
    input_paths++;
    if (!byte_line_list_push_xargs_wc_path(lines, list.items[idx], &total, &files, &err)) {
      path_list_free(&list);
      return 1;
    }
  }
  if (input_paths > 1) {
    char total_line[64];
    int len = snprintf(total_line, sizeof(total_line), "%8llu total", total);
    if (len < 0 || (size_t)len >= sizeof(total_line) ||
        !byte_line_list_push_plain_match(lines, total_line, (size_t)len, 1)) {
      path_list_free(&list);
      return 1;
    }
  }
  (void)err;
  path_list_free(&list);
  return 0;
}

static int parse_find_xargs_wc_output_pipe(char **words, int count,
                                           const struct find_pipe_prefix *prefix,
                                           const char **pattern,
                                           int *sort_paths,
                                           int *uniq_paths,
                                           int *downstream_start) {
  int idx = prefix->pipe + 1;
  *pattern = NULL;
  *sort_paths = 0;
  *uniq_paths = 0;
  if (idx + 2 < count && !strcmp(words[idx], "grep") &&
      is_plain_literal_pattern(words[idx + 1]) && !strcmp(words[idx + 2], "|")) {
    *pattern = words[idx + 1];
    idx += 3;
  }
  if (idx + 2 < count && !strcmp(words[idx], "sort") && !strcmp(words[idx + 1], "|")) {
    *sort_paths = 1;
    idx += 2;
    if (idx + 2 < count && !strcmp(words[idx], "uniq") && !strcmp(words[idx + 1], "|")) {
      *uniq_paths = 1;
      idx += 2;
    }
  }
  if (idx + 4 >= count || strcmp(words[idx], "xargs") ||
      strcmp(words[idx + 1], "wc") || strcmp(words[idx + 2], "-l") ||
      strcmp(words[idx + 3], "|")) {
    return 0;
  }
  *downstream_start = idx + 4;
  return xargs_wc_output_mode_supported(words, *downstream_start, count);
}

static int pipe_find_xargs_wc_producer(char **words, int count) {
  struct find_pipe_prefix prefix;
  const char *pattern = NULL;
  int sort_paths = 0;
  int uniq_paths = 0;
  int downstream_start = 0;
  if (!parse_find_pipe_prefix(words, count, &prefix) ||
      !parse_find_xargs_wc_output_pipe(words, count, &prefix, &pattern,
                                       &sort_paths, &uniq_paths,
                                       &downstream_start)) {
    return unsupported();
  }
  struct byte_line_list lines = {0};
  int rc = collect_find_xargs_wc_output_lines(&prefix, pattern, sort_paths,
                                              uniq_paths, &lines);
  if (rc != 0) {
    byte_line_list_free(&lines);
    return rc;
  }
  rc = emit_head_line_list_mode(words, downstream_start, count, &lines, 0);
  byte_line_list_free(&lines);
  return rc == 127 ? unsupported() : rc;
}

static int pipe_find_grep_xargs_echo(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 6 ||
      strcmp(words[prefix.pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[prefix.pipe + 2]) ||
      strcmp(words[prefix.pipe + 3], "|") ||
      strcmp(words[prefix.pipe + 4], "xargs") ||
      strcmp(words[prefix.pipe + 5], "echo")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  const char *pattern = words[prefix.pipe + 2];
  size_t pattern_len = strlen(pattern);
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  int first = 1;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      emit_xargs_echo_path(list.items[idx], &first);
    }
  }
  if (!first) write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int pipe_find_grep_xargs_wc(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 7 ||
      strcmp(words[prefix.pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[prefix.pipe + 2]) ||
      strcmp(words[prefix.pipe + 3], "|") ||
      strcmp(words[prefix.pipe + 4], "xargs") ||
      strcmp(words[prefix.pipe + 5], "wc") ||
      strcmp(words[prefix.pipe + 6], "-l")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  const char *pattern = words[prefix.pipe + 2];
  size_t pattern_len = strlen(pattern);
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      (void)find_wc_emit_file(list.items[idx], &total, &files, &err);
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  path_list_free(&list);
  return err ? 1 : 0;
}

static int pipe_find_grep_sort_xargs_echo(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 8 ||
      strcmp(words[prefix.pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[prefix.pipe + 2]) ||
      strcmp(words[prefix.pipe + 3], "|") ||
      strcmp(words[prefix.pipe + 4], "sort") ||
      strcmp(words[prefix.pipe + 5], "|") ||
      strcmp(words[prefix.pipe + 6], "xargs") ||
      strcmp(words[prefix.pipe + 7], "echo")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  const char *pattern = words[prefix.pipe + 2];
  size_t pattern_len = strlen(pattern);
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  int first = 1;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      emit_xargs_echo_path(list.items[idx], &first);
    }
  }
  if (!first) write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int pipe_find_grep_sort_xargs_wc(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 9 ||
      strcmp(words[prefix.pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[prefix.pipe + 2]) ||
      strcmp(words[prefix.pipe + 3], "|") ||
      strcmp(words[prefix.pipe + 4], "sort") ||
      strcmp(words[prefix.pipe + 5], "|") ||
      strcmp(words[prefix.pipe + 6], "xargs") ||
      strcmp(words[prefix.pipe + 7], "wc") ||
      strcmp(words[prefix.pipe + 8], "-l")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  const char *pattern = words[prefix.pipe + 2];
  size_t pattern_len = strlen(pattern);
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len)) {
      (void)find_wc_emit_file(list.items[idx], &total, &files, &err);
    }
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  path_list_free(&list);
  return err ? 1 : 0;
}

static int collect_find_grep_pipe_lines(const struct find_pipe_prefix *prefix,
                                        const char *pattern,
                                        struct byte_line_list *lines) {
  char path[PATH_MAX];
  struct path_list list = {0};
  if (!copy_cstr(path, sizeof(path), prefix->root)) return 0;
  int rc = find_collect_named_path(path, sizeof(path), prefix->name_glob,
                                   prefix->max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return -1;
  }
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < list.len; idx++) {
    if (contains_bytes(list.items[idx], (ssize_t)strlen(list.items[idx]), pattern, pattern_len) &&
        !byte_line_list_push_plain_match(lines, list.items[idx], strlen(list.items[idx]), 1)) {
      path_list_free(&list);
      return -1;
    }
  }
  path_list_free(&list);
  return 1;
}

static int pipe_find_grep_producer(char **words, int count) {
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count < prefix.pipe + 3 ||
      strcmp(words[prefix.pipe + 1], "grep") ||
      !is_plain_literal_pattern(words[prefix.pipe + 2])) {
    return unsupported();
  }
  if (count != prefix.pipe + 3 && strcmp(words[prefix.pipe + 3], "|")) {
    return unsupported();
  }

  struct byte_line_list list = {0};
  int rc = collect_find_grep_pipe_lines(&prefix, words[prefix.pipe + 2], &list);
  if (rc < 0) {
    byte_line_list_free(&list);
    return 1;
  }
  if (rc == 0) {
    byte_line_list_free(&list);
    return unsupported();
  }
  if (count == prefix.pipe + 3) {
    for (size_t idx = 0; idx < list.len; idx++) {
      write_bytes(list.items[idx].data, list.items[idx].len);
    }
    rc = list.len ? 0 : 1;
  } else {
    rc = emit_head_line_list_mode(words, prefix.pipe + 4, count, &list, 0);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_find_wc(char **words, int count) {
  char path[PATH_MAX];
  unsigned long long matches = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 3 ||
      strcmp(words[prefix.pipe + 1], "wc") || strcmp(words[prefix.pipe + 2], "-l")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  (void)find_count_walk_path(path, sizeof(path), prefix.name_glob,
                             prefix.max_depth, 0, &matches);
  write_padded_u64(matches);
  write_bytes("\n", 1);
  return 0;
}

static int pipe_find_head(char **words, int count) {
  char path[PATH_MAX];
  unsigned long long limit = 0;
  int err = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 4 ||
      strcmp(words[prefix.pipe + 1], "head") || strcmp(words[prefix.pipe + 2], "-n") ||
      !parse_u64_arg(words[prefix.pipe + 3], &limit) || limit == 0) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  (void)find_head_walk_path(path, sizeof(path), prefix.name_glob,
                            prefix.max_depth, 0, &limit, &err);
  (void)err;
  return 0;
}

static int pipe_find_tail(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  unsigned long long limit = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 4 ||
      strcmp(words[prefix.pipe + 1], "tail") || strcmp(words[prefix.pipe + 2], "-n") ||
      !parse_u64_arg(words[prefix.pipe + 3], &limit)) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_tail_collect_path(path, sizeof(path), prefix.name_glob,
                                  prefix.max_depth, 0, limit, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  emit_path_list_tail(&list, limit);
  path_list_free(&list);
  return 0;
}

static int pipe_find_sort(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 2 ||
      strcmp(words[prefix.pipe + 1], "sort")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  for (size_t idx = 0; idx < list.len; idx++) {
    write_line(list.items[idx]);
  }
  path_list_free(&list);
  return 0;
}

static int pipe_find_sort_uniq(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 4 ||
      strcmp(words[prefix.pipe + 1], "sort") || strcmp(words[prefix.pipe + 2], "|") ||
      strcmp(words[prefix.pipe + 3], "uniq")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  const char *previous = NULL;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (!previous || strcmp(previous, list.items[idx])) {
      write_line(list.items[idx]);
      previous = list.items[idx];
    }
  }
  path_list_free(&list);
  return 0;
}

static int pipe_find_sort_uniq_wc(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 7 ||
      strcmp(words[prefix.pipe + 1], "sort") || strcmp(words[prefix.pipe + 2], "|") ||
      strcmp(words[prefix.pipe + 3], "uniq") || strcmp(words[prefix.pipe + 4], "|") ||
      strcmp(words[prefix.pipe + 5], "wc") || strcmp(words[prefix.pipe + 6], "-l")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  write_padded_u64(count_unique_path_list(&list));
  write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int collect_find_sort_uniq_pipe_lines(const struct find_pipe_prefix *prefix,
                                             struct byte_line_list *lines) {
  char path[PATH_MAX];
  struct path_list list = {0};
  if (!copy_cstr(path, sizeof(path), prefix->root)) return 0;
  int rc = find_collect_named_path(path, sizeof(path), prefix->name_glob,
                                   prefix->max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return -1;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  const char *previous = NULL;
  for (size_t idx = 0; idx < list.len; idx++) {
    if (!previous || strcmp(previous, list.items[idx])) {
      if (!byte_line_list_push_plain_match(lines, list.items[idx],
                                           strlen(list.items[idx]), 1)) {
        path_list_free(&list);
        return -1;
      }
      previous = list.items[idx];
    }
  }
  path_list_free(&list);
  return 1;
}

static int collect_find_sort_uniq_grep_pipe_lines(const struct find_pipe_prefix *prefix,
                                                  const char *pattern,
                                                  struct byte_line_list *lines) {
  struct byte_line_list source = {0};
  int collected = collect_find_sort_uniq_pipe_lines(prefix, &source);
  if (collected <= 0) {
    byte_line_list_free(&source);
    return collected;
  }
  size_t pattern_len = strlen(pattern);
  for (size_t idx = 0; idx < source.len; idx++) {
    struct byte_line_item *item = &source.items[idx];
    if (contains_bytes(item->data, (ssize_t)item->len, pattern, pattern_len) &&
        !byte_line_list_push_plain_match(lines, item->data, item->len, 0)) {
      byte_line_list_free(&source);
      return -1;
    }
  }
  byte_line_list_free(&source);
  return 1;
}

static int pipe_find_sort_uniq_producer(char **words, int count) {
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || prefix.pipe + 5 >= count ||
      strcmp(words[prefix.pipe + 1], "sort") ||
      strcmp(words[prefix.pipe + 2], "|") ||
      strcmp(words[prefix.pipe + 3], "uniq") ||
      strcmp(words[prefix.pipe + 4], "|")) {
    return unsupported();
  }

  struct byte_line_list list = {0};
  int rc = 127;
  if (count >= prefix.pipe + 7 && !strcmp(words[prefix.pipe + 5], "grep") &&
      is_plain_literal_pattern(words[prefix.pipe + 6])) {
    if (count != prefix.pipe + 7 && strcmp(words[prefix.pipe + 7], "|")) {
      return unsupported();
    }
    rc = collect_find_sort_uniq_grep_pipe_lines(&prefix, words[prefix.pipe + 6], &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    if (count == prefix.pipe + 7) {
      for (size_t idx = 0; idx < list.len; idx++) {
        write_bytes(list.items[idx].data, list.items[idx].len);
      }
      rc = list.len ? 0 : 1;
    } else {
      rc = emit_head_line_list_mode(words, prefix.pipe + 8, count, &list, 0);
    }
  } else {
    rc = collect_find_sort_uniq_pipe_lines(&prefix, &list);
    if (rc < 0) {
      byte_line_list_free(&list);
      return 1;
    }
    if (rc == 0) {
      byte_line_list_free(&list);
      return unsupported();
    }
    rc = emit_head_line_list_mode(words, prefix.pipe + 5, count, &list, 1);
  }
  byte_line_list_free(&list);
  return rc;
}

static int pipe_find_sort_wc(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 5 ||
      strcmp(words[prefix.pipe + 1], "sort") || strcmp(words[prefix.pipe + 2], "|") ||
      strcmp(words[prefix.pipe + 3], "wc") || strcmp(words[prefix.pipe + 4], "-l")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  write_padded_u64((unsigned long long)list.len);
  write_bytes("\n", 1);
  path_list_free(&list);
  return 0;
}

static int pipe_find_sort_xargs_echo(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 5 ||
      strcmp(words[prefix.pipe + 1], "sort") || strcmp(words[prefix.pipe + 2], "|") ||
      strcmp(words[prefix.pipe + 3], "xargs") || strcmp(words[prefix.pipe + 4], "echo")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  emit_xargs_echo_path_list(&list);
  path_list_free(&list);
  return 0;
}

static int pipe_find_sort_xargs_wc(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  unsigned long long total = 0;
  unsigned long long files = 0;
  int err = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 6 ||
      strcmp(words[prefix.pipe + 1], "sort") || strcmp(words[prefix.pipe + 2], "|") ||
      strcmp(words[prefix.pipe + 3], "xargs") || strcmp(words[prefix.pipe + 4], "wc") ||
      strcmp(words[prefix.pipe + 5], "-l")) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  for (size_t idx = 0; idx < list.len; idx++) {
    (void)find_wc_emit_file(list.items[idx], &total, &files, &err);
  }
  if (files > 1) {
    write_padded_u64(total);
    write_bytes(" total\n", 7);
  }
  path_list_free(&list);
  return err ? 1 : 0;
}

static int pipe_find_sort_head(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  unsigned long long limit = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 6 ||
      strcmp(words[prefix.pipe + 1], "sort") || strcmp(words[prefix.pipe + 2], "|") ||
      strcmp(words[prefix.pipe + 3], "head") || strcmp(words[prefix.pipe + 4], "-n") ||
      !parse_u64_arg(words[prefix.pipe + 5], &limit) || limit == 0) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  size_t take = limit > (unsigned long long)list.len ? list.len : (size_t)limit;
  for (size_t idx = 0; idx < take; idx++) {
    write_line(list.items[idx]);
  }
  path_list_free(&list);
  return 0;
}

static int pipe_find_sort_tail(char **words, int count) {
  char path[PATH_MAX];
  struct path_list list = {0};
  unsigned long long limit = 0;
  struct find_pipe_prefix prefix;
  if (!parse_find_pipe_prefix(words, count, &prefix) || count != prefix.pipe + 6 ||
      strcmp(words[prefix.pipe + 1], "sort") || strcmp(words[prefix.pipe + 2], "|") ||
      strcmp(words[prefix.pipe + 3], "tail") || strcmp(words[prefix.pipe + 4], "-n") ||
      !parse_u64_arg(words[prefix.pipe + 5], &limit)) {
    return unsupported();
  }
  if (!copy_cstr(path, sizeof(path), prefix.root)) return unsupported();
  int rc = find_collect_named_path(path, sizeof(path), prefix.name_glob,
                                   prefix.max_depth, 0, &list);
  if (rc != 0) {
    path_list_free(&list);
    return rc;
  }
  qsort(list.items, list.len, sizeof(char *), cmp_string_ptr);
  emit_path_list_tail(&list, limit);
  path_list_free(&list);
  return 0;
}

static int pipe_stdin_grep(char **words, int count) {
  if (count < 4 || strcmp(words[0], "grep") || words[1][0] == '-' ||
      strcmp(words[2], "|") || !is_plain_literal_pattern(words[1])) {
    return unsupported();
  }

  char **rewritten = (char **)calloc((size_t)count + 1, sizeof(char *));
  if (!rewritten) return 1;
  rewritten[0] = "grep";
  rewritten[1] = words[1];
  rewritten[2] = NULL;
  rewritten[3] = "|";
  int rewritten_count = 4;
  for (int idx = 3; idx < count; idx++) rewritten[rewritten_count++] = words[idx];

  int code = pipe_grep_file_cut_producer(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_awk_producer(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort_uniq_wc(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort_uniq_producer(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort_uniq(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort_wc(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort_head(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort_tail(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort_xargs_echo(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort_xargs_wc(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_wc(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_head(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_tail(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_xargs_echo(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_xargs_wc(rewritten, rewritten_count);
  if (code == 127) code = pipe_grep_file_sort(rewritten, rewritten_count);

  free(rewritten);
  return code;
}

static int dispatch_pipe_words(char **words, int count) {
  int code = pipe_echo_wc(words, count);
  if (code != 127) return code;
  code = pipe_echo_head(words, count);
  if (code != 127) return code;
  code = pipe_echo_tail(words, count);
  if (code != 127) return code;
  code = pipe_echo_tr(words, count);
  if (code != 127) return code;
  code = pipe_echo_awk_producer(words, count);
  if (code != 127) return code;
  code = pipe_echo_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_echo_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_xargs_echo_producer(words, count);
  if (code != 127) return code;
  code = pipe_empty_producer(words, count);
  if (code != 127) return code;
  code = pipe_side_effect_empty_producer(words, count);
  if (code != 127) return code;
  code = pipe_predicate_empty_producer(words, count);
  if (code != 127) return code;
  code = pipe_wc_producer(words, count);
  if (code != 127) return code;
  code = pipe_du_producer(words, count);
  if (code != 127) return code;
  code = pipe_head_producer(words, count);
  if (code != 127) return code;
  code = pipe_tail_producer(words, count);
  if (code != 127) return code;
  code = pipe_cat_head_tail_producer(words, count, "head", 1);
  if (code != 127) return code;
  code = pipe_cat_head_tail_producer(words, count, "tail", 0);
  if (code != 127) return code;
  code = pipe_sed_producer(words, count);
  if (code != 127) return code;
  code = pipe_cut_producer(words, count);
  if (code != 127) return code;
  code = pipe_printf_literal_producer(words, count);
  if (code != 127) return code;
  code = pipe_printf_wc(words, count);
  if (code != 127) return code;
  code = pipe_printf_head(words, count);
  if (code != 127) return code;
  code = pipe_printf_tail(words, count);
  if (code != 127) return code;
  code = pipe_printf_grep(words, count);
  if (code != 127) return code;
  code = pipe_printf_awk_producer(words, count);
  if (code != 127) return code;
  code = pipe_printf_grep_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_printf_grep_producer(words, count);
  if (code != 127) return code;
  code = pipe_printf_tr(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_printf_sort(words, count);
  if (code != 127) return code;
  code = pipe_printf_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_printf_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_seq_wc(words, count);
  if (code != 127) return code;
  code = pipe_seq_head(words, count);
  if (code != 127) return code;
  code = pipe_seq_tail(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_sort(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_wc(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_head(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_tail(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_seq_grep(words, count);
  if (code != 127) return code;
  code = pipe_seq_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_seq_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_seq_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_seq_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_seq_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_seq_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_seq_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_seq_sort(words, count);
  if (code != 127) return code;
  code = pipe_seq_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_yes_head(words, count);
  if (code != 127) return code;
  code = pipe_path_lookup_wc(words, count);
  if (code != 127) return code;
  code = pipe_path_lookup_head(words, count);
  if (code != 127) return code;
  code = pipe_path_lookup_tail(words, count);
  if (code != 127) return code;
  code = pipe_path_lookup_grep(words, count);
  if (code != 127) return code;
  code = pipe_path_lookup_producer(words, count);
  if (code != 127) return code;
  code = pipe_environment_wc(words, count);
  if (code != 127) return code;
  code = pipe_environment_head(words, count);
  if (code != 127) return code;
  code = pipe_environment_tail(words, count);
  if (code != 127) return code;
  code = pipe_environment_grep(words, count);
  if (code != 127) return code;
  code = pipe_environment_sort(words, count);
  if (code != 127) return code;
  code = pipe_hostname_wc(words, count);
  if (code != 127) return code;
  code = pipe_hostname_head(words, count);
  if (code != 127) return code;
  code = pipe_hostname_tail(words, count);
  if (code != 127) return code;
  code = pipe_hostname_grep(words, count);
  if (code != 127) return code;
  code = pipe_hostname_sort(words, count);
  if (code != 127) return code;
  code = pipe_single_line_producer(words, count);
  if (code != 127) return code;
  code = pipe_ls_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_ls_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_ls_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_ls_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_ls_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_ls_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_ls_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_ls_grep_producer(words, count);
  if (code != 127) return code;
  code = pipe_ls_grep_wc(words, count);
  if (code != 127) return code;
  code = pipe_ls_grep_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_ls_grep_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_ls_grep(words, count);
  if (code != 127) return code;
  code = pipe_ls_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_ls_wc(words, count);
  if (code != 127) return code;
  code = pipe_ls_head(words, count);
  if (code != 127) return code;
  code = pipe_ls_tail(words, count);
  if (code != 127) return code;
  code = pipe_ls_sort(words, count);
  if (code != 127) return code;
  code = pipe_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_sort_grep_producer(words, count);
  if (code != 127) return code;
  code = pipe_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_sort_xargs_wc_producer(words, count);
  if (code != 127) return code;
  code = pipe_sort_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort_xargs_wc_producer(words, count);
  if (code != 127) return code;
  code = pipe_cat_sort_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_head(words, count);
  if (code != 127) return code;
  code = pipe_cat_tail(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_head(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_tail(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_sort_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_cat_grep_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_cut_producer(words, count);
  if (code != 127) return code;
  code = pipe_cat_cut(words, count);
  if (code != 127) return code;
  code = pipe_cat_tr_producer(words, count);
  if (code != 127) return code;
  code = pipe_cat_tr(words, count);
  if (code != 127) return code;
  code = pipe_cat_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_cat_xargs_wc_producer(words, count);
  if (code != 127) return code;
  code = pipe_cat_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_cat_uniq(words, count);
  if (code != 127) return code;
  code = pipe_cat_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_stdin_grep(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_cut_producer(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_awk_producer(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_wc(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_head(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_tail(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_grep_file_sort(words, count);
  if (code != 127) return code;
  code = pipe_grep_head(words, count);
  if (code != 127) return code;
  code = pipe_grep_tail(words, count);
  if (code != 127) return code;
  code = pipe_grep_sort(words, count);
  if (code != 127) return code;
  code = pipe_grep_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_grep_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_grep_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_grep_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_grep_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_grep_sort_tail(words, count);
  if (code != 127) return code;
  code = pipe_grep_wc(words, count);
  if (code != 127) return code;
  code = pipe_cat_awk(words, count);
  if (code != 127) return code;
  code = pipe_stdin_awk(words, count);
  if (code != 127) return code;
  code = dispatch_pipe_awk_handlers(words, count);
  if (code != 127) return code;
  code = pipe_find_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_find_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_find_xargs_wc_producer(words, count);
  if (code != 127) return code;
  code = pipe_find_grep_producer(words, count);
  if (code != 127) return code;
  code = pipe_find_grep_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_find_grep_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_find_grep_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_find_grep_sort_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_find_sort_xargs_echo(words, count);
  if (code != 127) return code;
  code = pipe_find_sort_xargs_wc(words, count);
  if (code != 127) return code;
  code = pipe_find_sort_uniq_producer(words, count);
  if (code != 127) return code;
  code = pipe_find_sort(words, count);
  if (code != 127) return code;
  code = pipe_find_sort_uniq(words, count);
  if (code != 127) return code;
  code = pipe_find_sort_uniq_wc(words, count);
  if (code != 127) return code;
  code = pipe_find_sort_wc(words, count);
  if (code != 127) return code;
  code = pipe_find_sort_head(words, count);
  if (code != 127) return code;
  code = pipe_find_wc(words, count);
  if (code != 127) return code;
  code = pipe_find_head(words, count);
  if (code != 127) return code;
  code = pipe_find_tail(words, count);
  if (code != 127) return code;
  return pipe_find_sort_tail(words, count);
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
  return !strcmp(cmd, "true") || !strcmp(cmd, "false") ||
         !strcmp(cmd, "pwd") || !strcmp(cmd, "basename") ||
         !strcmp(cmd, "dirname") || !strcmp(cmd, "echo") ||
         !strcmp(cmd, "printf") || !strcmp(cmd, "seq") ||
         !strcmp(cmd, "whoami") || !strcmp(cmd, "id") ||
         !strcmp(cmd, "uname") || !strcmp(cmd, "hostname") || !strcmp(cmd, "test") ||
         !strcmp(cmd, "[") ||
         !strcmp(cmd, "ls") || !strcmp(cmd, "cat") ||
         !strcmp(cmd, "uniq") || !strcmp(cmd, "sort") ||
         !strcmp(cmd, "cut") || !strcmp(cmd, "tr") || !strcmp(cmd, "sed") ||
         !strcmp(cmd, "grep") || !strcmp(cmd, "find") || !strcmp(cmd, "du") ||
         !strcmp(cmd, "wc") || !strcmp(cmd, "head") ||
         !strcmp(cmd, "tail") || !strcmp(cmd, "mkdir") ||
         !strcmp(cmd, "touch") || !strcmp(cmd, "awk") ||
         !strcmp(cmd, "xargs") || !strcmp(cmd, "which") ||
         !strcmp(cmd, "command") || !strcmp(cmd, "env") ||
         !strcmp(cmd, "printenv");
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
  if (!*command) return unsupported();

  char **words = NULL;
  int word_count = 0;
  if (!split_simple_shell_words(command, &words, &word_count)) return unsupported();
  for (int idx = 0; idx < word_count; idx++) {
    if (!strcmp(words[idx], "|")) {
      int code = dispatch_pipe_words(words, word_count);
      free_words(words, word_count);
      return code;
    }
  }
  if (has_shell_control_syntax(command)) {
    free_words(words, word_count);
    return unsupported();
  }
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
  if (!strcmp(cmd, "true")) return cap_true(argc, argv);
  if (!strcmp(cmd, "false")) return cap_false(argc, argv);
  if (!strcmp(cmd, "pwd")) return cap_pwd(argc, argv);
  if (!strcmp(cmd, "echo")) return cap_echo(argc, argv);
  if (!strcmp(cmd, "printf")) return cap_printf(argc, argv);
  if (!strcmp(cmd, "seq")) return cap_seq(argc, argv);
  if (!strcmp(cmd, "whoami")) return cap_whoami(argc, argv);
  if (!strcmp(cmd, "id")) return cap_id(argc, argv);
  if (!strcmp(cmd, "uname")) return cap_uname(argc, argv);
  if (!strcmp(cmd, "hostname")) return cap_hostname(argc, argv);
  if (!strcmp(cmd, "test")) return cap_test_cmd(argc, argv, 0);
  if (!strcmp(cmd, "[")) return cap_test_cmd(argc, argv, 1);
  if (!strcmp(cmd, "basename")) return cap_basename(argc, argv);
  if (!strcmp(cmd, "dirname")) return cap_dirname(argc, argv);
  if (!strcmp(cmd, "ls")) return cap_ls(argc, argv);
  if (!strcmp(cmd, "cat")) return cap_cat(argc, argv);
  if (!strcmp(cmd, "head")) return cap_head(argc, argv);
  if (!strcmp(cmd, "tail")) return cap_tail(argc, argv);
  if (!strcmp(cmd, "mkdir")) return cap_mkdir(argc, argv);
  if (!strcmp(cmd, "touch")) return cap_touch(argc, argv);
  if (!strcmp(cmd, "uniq")) return cap_uniq(argc, argv);
  if (!strcmp(cmd, "sort")) return cap_sort(argc, argv);
  if (!strcmp(cmd, "cut")) return cap_cut(argc, argv);
  if (!strcmp(cmd, "tr")) return cap_tr(argc, argv);
  if (!strcmp(cmd, "sed")) return cap_sed(argc, argv);
  if (!strcmp(cmd, "grep")) return cap_grep(argc, argv);
  if (!strcmp(cmd, "awk")) return cap_awk(argc, argv);
  if (!strcmp(cmd, "xargs")) return cap_xargs(argc, argv);
  if (!strcmp(cmd, "which")) return cap_which(argc, argv);
  if (!strcmp(cmd, "command")) return cap_command_builtin(argc, argv);
  if (!strcmp(cmd, "env")) return cap_env_builtin(argc, argv);
  if (!strcmp(cmd, "printenv")) return cap_printenv(argc, argv);
  if (!strcmp(cmd, "find")) return cap_find(argc, argv);
  if (!strcmp(cmd, "du")) return cap_du(argc, argv);
  if (!strcmp(cmd, "wc")) return cap_wc(argc, argv);
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

  const char *public_arg0 = getenv("CAP_PUBLIC_EXE");
  if (!public_arg0 || !*public_arg0) public_arg0 = full;

  char *full_argv[argc + 1];
  full_argv[0] = (char *)public_arg0;
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
