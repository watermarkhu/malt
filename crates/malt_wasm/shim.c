/* Freestanding C shim for the tree-sitter C parser compiled to wasm32.
 *
 * The tree-sitter-matlab parser/scanner need a small libc subset. When the
 * `cc` crate compiles them with the WASI sysroot, the linker pulls in the
 * WASI libc, which imports `malloc`/`free`/`printf` etc. from `env` — not
 * available in a browser.
 *
 * Instead we provide those symbols ourselves, backed by the wasm-bindgen
 * allocator and no-op stubs for the I/O paths tree-sitter only uses for
 * debugging graphs. This keeps the resulting wasm free of `env` imports.
 *
 * The `cc` build for this crate includes this file via the `wasm-cc` feature
 * (see `build.rs`).
 */

#include <stddef.h>

/* wasm-bindgen exports the allocator with these exact signatures
 * (see wasm-bindgen/src/rt/mod.rs):
 *   __wbindgen_malloc(size, align) -> ptr
 *   __wbindgen_free(ptr, size, align)
 * They are satisfied at link time by the wasm-bindgen runtime. */
void *__wbindgen_malloc(unsigned long size, unsigned long align);
void __wbindgen_free(void *ptr, unsigned long size, unsigned long align);

void *malloc(size_t size) { return __wbindgen_malloc((unsigned long)size, 1); }
void *calloc(size_t n, size_t size) {
  void *p = __wbindgen_malloc((unsigned long)(n * size), 1);
  if (p != 0) {
    char *q = p;
    for (size_t i = 0; i < n * size; i++) q[i] = 0;
  }
  return p;
}
void *realloc(void *ptr, size_t size) {
  /* wasm-bindgen has no realloc; allocate + copy is the safest stub. */
  void *p = __wbindgen_malloc((unsigned long)size, 1);
  if (p != 0 && ptr != 0) {
    char *src = ptr;
    char *dst = p;
    for (size_t i = 0; i < size; i++) dst[i] = src[i];
  }
  return p;
}
void free(void *ptr) { __wbindgen_free(ptr, 0, 1); }

/* --- no-op / minimal stubs for functions tree-sitter only uses for
 *     debugging or that werei does not provide in a browser ------------ */

int fprintf(void *stream, const char *fmt, ...) { (void)stream; (void)fmt; return 0; }
int snprintf(char *buf, size_t size, const char *fmt, ...) {
  (void)buf; (void)size; (void)fmt; return 0;
}
int vsnprintf(char *buf, size_t size, const char *fmt, void *ap) {
  (void)buf; (void)size; (void)fmt; (void)ap; return 0;
}
int fwrite(const void *ptr, size_t size, size_t n, void *stream) {
  (void)ptr; (void)size; (void)n; (void)stream; return 0;
}
int fputc(int c, void *stream) { (void)c; (void)stream; return c; }
int fclose(void *stream) { (void)stream; return 0; }
int fdopen(int fd, const char *mode) { (void)fd; (void)mode; return 0; }
void abort(void) { for (;;) {} }
void __assert_fail(const char *a, const char *b, int c, const char *d) {
  (void)a; (void)b; (void)c; (void)d; for (;;) {}
}
int strncmp(const char *a, const char *b, size_t n) {
  for (size_t i = 0; i < n; i++) {
    if (a[i] != b[i]) return (unsigned char)a[i] - (unsigned char)b[i];
    if (a[i] == 0) return 0;
  }
  return 0;
}
int clock_gettime(int clk_id, void *ts) { (void)clk_id; (void)ts; return 0; }

/* --- wctype / ctype helpers used by the MATLAB scanner ------------------ */

int iswspace(int c) {
  return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v';
}
int iswalpha(int c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}
int iswdigit(int c) { return c >= '0' && c <= '9'; }

/* --- string.h helpers (may be provided by compiler builtins, but define
 *     them explicitly to be safe) --------------------------------------- */

size_t strlen(const char *s) {
  const char *p = s;
  while (*p) p++;
  return (size_t)(p - s);
}
int strcmp(const char *a, const char *b) {
  while (*a && *a == *b) { a++; b++; }
  return (unsigned char)*a - (unsigned char)*b;
}
void *memcpy(void *dst, const void *src, size_t n) {
  char *d = dst;
  const char *s = src;
  for (size_t i = 0; i < n; i++) d[i] = s[i];
  return dst;
}
void *memmove(void *dst, const void *src, size_t n) {
  char *d = dst;
  const char *s = src;
  if (d <= s) {
    for (size_t i = 0; i < n; i++) d[i] = s[i];
  } else {
    for (size_t i = n; i > 0; i--) d[i - 1] = s[i - 1];
  }
  return dst;
}
void *memset(void *dst, int c, size_t n) {
  unsigned char *d = dst;
  for (size_t i = 0; i < n; i++) d[i] = (unsigned char)c;
  return dst;
}
