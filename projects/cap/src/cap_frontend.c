// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
// Minimal public cap dispatcher.
//
// On macOS arm64 this file builds without C runtime start files and uses direct
// Darwin syscalls for tiny commands. That keeps direct stdout-discard paths
// below Rust process overhead while heavier replacements delegate to sibling
// cap-fast. Other platforms use a normal C entrypoint with the same command
// shape.

#if defined(__APPLE__) && defined(__aarch64__)

#include <mach-o/dyld.h>
#include <fcntl.h>
#include <stddef.h>
#include <sys/stat.h>
#include <sys/types.h>

#ifndef PATH_MAX
#define PATH_MAX 1024
#endif

static long syscall1(long n, long a) {
  register long x0 asm("x0") = a;
  register long x16 asm("x16") = n;
  asm volatile(
      "svc #0x80\n"
      "b.cc 1f\n"
      "neg %0, %0\n"
      "1:"
      : "+r"(x0)
      : "r"(x16)
      : "cc", "memory");
  return x0;
}

static long syscall2(long n, long a, long b) {
  register long x0 asm("x0") = a;
  register long x1 asm("x1") = b;
  register long x16 asm("x16") = n;
  asm volatile(
      "svc #0x80\n"
      "b.cc 1f\n"
      "neg %0, %0\n"
      "1:"
      : "+r"(x0)
      : "r"(x1), "r"(x16)
      : "cc", "memory");
  return x0;
}

static long syscall3(long n, long a, long b, long c) {
  register long x0 asm("x0") = a;
  register long x1 asm("x1") = b;
  register long x2 asm("x2") = c;
  register long x16 asm("x16") = n;
  asm volatile(
      "svc #0x80\n"
      "b.cc 1f\n"
      "neg %0, %0\n"
      "1:"
      : "+r"(x0)
      : "r"(x1), "r"(x2), "r"(x16)
      : "cc", "memory");
  return x0;
}

static size_t c_len(const char *s) {
  size_t n = 0;
  while (s[n]) n++;
  return n;
}

static int c_eq(const char *a, const char *b) {
  while (*a && *a == *b) {
    a++;
    b++;
  }
  return *a == 0 && *b == 0;
}

static char *c_rchr(char *s, char c) {
  char *out = 0;
  while (*s) {
    if (*s == c) out = s;
    s++;
  }
  return out;
}

static void c_copy(char *dst, const char *src, size_t n) {
  for (size_t idx = 0; idx < n; idx++) dst[idx] = src[idx];
}

static void write_fd_all(long fd, const char *bytes, size_t len) {
  while (len > 0) {
    long written = syscall3(4, fd, (long)bytes, (long)len);
    if (written <= 0) return;
    bytes += (size_t)written;
    len -= (size_t)written;
  }
}

static void write_all(const char *bytes, size_t len) {
  write_fd_all(1, bytes, len);
}

static const char *err_text(long err) {
  switch (err) {
    case 2:
      return "No such file or directory";
    case 13:
      return "Permission denied";
    case 21:
      return "Is a directory";
    default:
      return "Input/output error";
  }
}

static void write_err_path(const char *cmd, const char *path, long err) {
  write_fd_all(2, cmd, c_len(cmd));
  write_fd_all(2, ": ", 2);
  write_fd_all(2, path, c_len(path));
  write_fd_all(2, ": ", 2);
  const char *msg = err_text(err);
  write_fd_all(2, msg, c_len(msg));
  write_fd_all(2, "\n", 1);
}

static int stdout_is_dev_null(void) {
  struct stat st;
  if (syscall2(339, 1, (long)&st) != 0) return 0;
  return S_ISCHR(st.st_mode) && st.st_rdev == 50331650;
}

static int cap_cat(long argc, char **argv) {
  char buf[8192];
  int exit_code = 0;
  int skip_regular_reads = stdout_is_dev_null();
  if (argc < 3) return 127;
  for (long idx = 2; idx < argc; idx++) {
    if (argv[idx][0] == '-') return 127;
    long fd = syscall3(5, (long)argv[idx], O_RDONLY, 0);
    if (fd < 0) {
      write_err_path("cat", argv[idx], -fd);
      exit_code = 1;
      continue;
    }
    if (skip_regular_reads) {
      struct stat st;
      if (syscall2(339, fd, (long)&st) == 0 && S_ISREG(st.st_mode)) {
        syscall1(6, fd);
        continue;
      }
    }
    for (;;) {
      long read_len = syscall3(3, fd, (long)buf, sizeof(buf));
      if (read_len == 0) break;
      if (read_len < 0) {
        write_err_path("cat", argv[idx], -read_len);
        exit_code = 1;
        break;
      }
      write_all(buf, (size_t)read_len);
    }
    syscall1(6, fd);
  }
  return exit_code;
}

static int ascii_space(char c) {
  return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' || c == '\f';
}

static int split_run_words(const char *command, char *buf, size_t buf_cap,
                           char **words, long max_words, long *out_count) {
  enum { NORMAL, SINGLE, DOUBLE } state = NORMAL;
  size_t out = 0;
  long count = 0;
  int in_token = 0;

  for (const char *p = command; *p; p++) {
    char ch = *p;
    if (state == NORMAL) {
      switch (ch) {
        case '\'':
          if (!in_token) {
            if (count >= max_words || out >= buf_cap) return 0;
            words[count++] = &buf[out];
            in_token = 1;
          }
          state = SINGLE;
          break;
        case '"':
          if (!in_token) {
            if (count >= max_words || out >= buf_cap) return 0;
            words[count++] = &buf[out];
            in_token = 1;
          }
          state = DOUBLE;
          break;
        case '\\':
          if (!p[1]) return 0;
          if (!in_token) {
            if (count >= max_words || out >= buf_cap) return 0;
            words[count++] = &buf[out];
            in_token = 1;
          }
          if (out + 1 >= buf_cap) return 0;
          buf[out++] = *++p;
          break;
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
          return 0;
        default:
          if (ascii_space(ch)) {
            if (in_token) {
              if (out >= buf_cap) return 0;
              buf[out++] = 0;
              in_token = 0;
            }
          } else {
            if (!in_token) {
              if (count >= max_words || out >= buf_cap) return 0;
              words[count++] = &buf[out];
              in_token = 1;
            }
            if (out + 1 >= buf_cap) return 0;
            buf[out++] = ch;
          }
          break;
      }
    } else if (state == SINGLE) {
      if (ch == '\'') {
        state = NORMAL;
      } else {
        if (out + 1 >= buf_cap) return 0;
        buf[out++] = ch;
      }
    } else {
      if (ch == '"') {
        state = NORMAL;
      } else if (ch == '\\') {
        if (!p[1] || out + 1 >= buf_cap) return 0;
        buf[out++] = *++p;
      } else if (ch == '$' || ch == '`') {
        return 0;
      } else {
        if (out + 1 >= buf_cap) return 0;
        buf[out++] = ch;
      }
    }
  }

  if (state != NORMAL) return 0;
  if (in_token) {
    if (out >= buf_cap) return 0;
    buf[out++] = 0;
  }
  *out_count = count;
  return 1;
}

static int cap_run_direct(long argc, char **argv) {
  char buf[4096];
  char *words[128];
  char *rewritten[130];
  long count = 0;
  if (argc != 3 || !c_eq(argv[1], "run")) return 127;
  if (!split_run_words(argv[2], buf, sizeof(buf), words, 128, &count)) return 127;
  if (count < 2 || !c_eq(words[0], "cat")) return 127;
  rewritten[0] = argv[0];
  for (long idx = 0; idx < count; idx++) rewritten[idx + 1] = words[idx];
  rewritten[count + 1] = 0;
  return cap_cat(count + 1, rewritten);
}

static int executable_path(char *buf, unsigned int cap, char *argv0) {
  if (c_rchr(argv0, '/')) {
    size_t len = c_len(argv0);
    if (len + 1 > cap) return 0;
    c_copy(buf, argv0, len + 1);
    return 1;
  }
  return _NSGetExecutablePath(buf, &cap) == 0;
}

static int exec_fast(long argc, char **argv) {
  char self[PATH_MAX];
  char fast[PATH_MAX];
  if (!executable_path(self, sizeof(self), argv[0])) return 127;
  char *slash = c_rchr(self, '/');
  if (!slash) return 127;
  size_t dir_len = (size_t)(slash - self);
  if (dir_len + 10 > sizeof(fast)) return 127;
  c_copy(fast, self, dir_len);
  c_copy(fast + dir_len, "/cap-fast", 10);

  char *fast_argv[argc + 1];
  fast_argv[0] = fast;
  for (long idx = 1; idx < argc; idx++) fast_argv[idx] = argv[idx];
  fast_argv[argc] = 0;
  char *empty_env[] = {0};
  syscall3(59, (long)fast, (long)fast_argv, (long)empty_env);
  return 127;
}

static int c_main(long argc, char **argv) {
  if (argc < 2) return 127;
  if (c_eq(argv[1], "run")) {
    int code = cap_run_direct(argc, argv);
    if (code != 127) return code;
  }
  if (c_eq(argv[1], "cat")) return cap_cat(argc, argv);
  return exec_fast(argc, argv);
}

void start(long argc, char **argv) {
  syscall1(1, c_main(argc, argv) & 255);
  for (;;) {}
}

#else

#include <fcntl.h>
#include <errno.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/types.h>
#include <unistd.h>

static const char *cap_base(const char *s) {
  const char *p = strrchr(s, '/');
  return p ? p + 1 : s;
}

static int unsupported(void) { return 127; }

static void write_all(const char *bytes, size_t len) {
  while (len > 0) {
    ssize_t written = write(1, bytes, len);
    if (written <= 0) return;
    bytes += (size_t)written;
    len -= (size_t)written;
  }
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
  write_fd_all(2, path, strlen(path));
  write_fd_all(2, ": ", 2);
  write_fd_all(2, strerror(err), strlen(strerror(err)));
  write_fd_all(2, "\n", 1);
}

static int copy_cstr(char *dst, size_t cap, const char *src) {
  size_t len = strlen(src);
  if (len + 1 > cap) return 0;
  memcpy(dst, src, len + 1);
  return 1;
}

static int stdout_is_dev_null(void) {
  struct stat out_st;
  struct stat null_st;
  return fstat(1, &out_st) == 0 && stat("/dev/null", &null_st) == 0 &&
         out_st.st_dev == null_st.st_dev && out_st.st_ino == null_st.st_ino;
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
      write_all(buf, (size_t)read_len);
    }
    close(fd);
  }
  return exit_code;
}

static int split_run_words(const char *command, char *buf, size_t buf_cap,
                           char **words, int max_words, int *out_count) {
  enum { NORMAL, SINGLE, DOUBLE } state = NORMAL;
  size_t out = 0;
  int count = 0;
  int in_token = 0;

  for (const char *p = command; *p; p++) {
    char ch = *p;
    if (state == NORMAL) {
      switch (ch) {
        case '\'':
          if (!in_token) {
            if (count >= max_words || out >= buf_cap) return 0;
            words[count++] = &buf[out];
            in_token = 1;
          }
          state = SINGLE;
          break;
        case '"':
          if (!in_token) {
            if (count >= max_words || out >= buf_cap) return 0;
            words[count++] = &buf[out];
            in_token = 1;
          }
          state = DOUBLE;
          break;
        case '\\':
          if (!p[1]) return 0;
          if (!in_token) {
            if (count >= max_words || out >= buf_cap) return 0;
            words[count++] = &buf[out];
            in_token = 1;
          }
          if (out + 1 >= buf_cap) return 0;
          buf[out++] = *++p;
          break;
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
          return 0;
        default:
          if (ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' ||
              ch == '\v' || ch == '\f') {
            if (in_token) {
              if (out >= buf_cap) return 0;
              buf[out++] = 0;
              in_token = 0;
            }
          } else {
            if (!in_token) {
              if (count >= max_words || out >= buf_cap) return 0;
              words[count++] = &buf[out];
              in_token = 1;
            }
            if (out + 1 >= buf_cap) return 0;
            buf[out++] = ch;
          }
          break;
      }
    } else if (state == SINGLE) {
      if (ch == '\'') {
        state = NORMAL;
      } else {
        if (out + 1 >= buf_cap) return 0;
        buf[out++] = ch;
      }
    } else {
      if (ch == '"') {
        state = NORMAL;
      } else if (ch == '\\') {
        if (!p[1] || out + 1 >= buf_cap) return 0;
        buf[out++] = *++p;
      } else if (ch == '$' || ch == '`') {
        return 0;
      } else {
        if (out + 1 >= buf_cap) return 0;
        buf[out++] = ch;
      }
    }
  }

  if (state != NORMAL) return 0;
  if (in_token) {
    if (out >= buf_cap) return 0;
    buf[out++] = 0;
  }
  *out_count = count;
  return 1;
}

static int cap_run_direct(int argc, char **argv) {
  char buf[4096];
  char *words[128];
  char *rewritten[130];
  int count = 0;
  if (argc != 3 || strcmp(argv[1], "run")) return unsupported();
  if (!split_run_words(argv[2], buf, sizeof(buf), words, 128, &count)) return unsupported();
  if (count < 2 || strcmp(words[0], "cat")) return unsupported();
  rewritten[0] = argv[0];
  for (int idx = 0; idx < count; idx++) rewritten[idx + 1] = words[idx];
  rewritten[count + 1] = NULL;
  return cap_cat(count + 1, rewritten);
}

static int dispatch_tiny(int argc, char **argv) {
  if (argc < 2) return unsupported();
  const char *cmd = cap_base(argv[1]);
  if (!strcmp(cmd, "run")) return cap_run_direct(argc, argv);
  if (!strcmp(cmd, "cat")) return cap_cat(argc, argv);
  return unsupported();
}

static int exec_fast(int argc, char **argv) {
  char fast[PATH_MAX];
  const char *slash = strrchr(argv[0], '/');
  if (slash) {
    size_t dir_len = (size_t)(slash - argv[0]);
    if (dir_len + strlen("/cap-fast") + 1 > sizeof(fast)) return 127;
    memcpy(fast, argv[0], dir_len);
    memcpy(fast + dir_len, "/cap-fast", strlen("/cap-fast") + 1);
  } else {
    if (!copy_cstr(fast, sizeof(fast), "cap-fast")) return 127;
  }

  char *fast_argv[argc + 1];
  fast_argv[0] = fast;
  for (int idx = 1; idx < argc; idx++) fast_argv[idx] = argv[idx];
  fast_argv[argc] = NULL;

  if (slash) {
    execv(fast, fast_argv);
  } else {
    execvp(fast, fast_argv);
  }
  return 127;
}

int main(int argc, char **argv) {
  int code = dispatch_tiny(argc, argv);
  if (code != 127) return code;
  return exec_fast(argc, argv);
}

#endif
// CODEGEN-END
