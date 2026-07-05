/*
 * C-клиент, слинкованный со ffi_smart_socket staticlib.
 */

#ifndef __USE_MINGW_ANSI_STDIO
#  define __USE_MINGW_ANSI_STDIO 0
#endif

#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include "ffi_smart_socket.h"

static void print_state(const char *label, const SmartSocket *s)
{
    int on = socket_is_on(s);
    const char *st = (on == 1) ? "on" : (on == 0) ? "off" : "(err)";
    printf("  %-24s state=%-6s power=%.1f W\n", label, st, socket_power(s));
}

static void check(const char *op, int rc)
{
    if (rc == 0)  return;
    if (rc == -1) { fprintf(stderr, "[%s] null ptr\n",   op); exit(1); }
    if (rc == -2) { fprintf(stderr, "[%s] op failed\n",  op); exit(1); }
    fprintf(stderr, "[%s] unknown code %d\n", op, rc); exit(1);
}

static int g_pass = 0, g_fail = 0;

#define ASSERT_INT(expr, want, msg) do { \
    int _v = (expr); \
    if (_v == (want)) { printf("  [OK]   %s\n", (msg)); g_pass++; } \
    else { printf("  [FAIL] %s: want %d got %d\n", (msg), (want), _v); g_fail++; } \
} while(0)

#define ASSERT_DBL(expr, want, eps, msg) do { \
    double _v = (expr); \
    if (fabs(_v - (want)) < (eps)) { printf("  [OK]   %s\n", (msg)); g_pass++; } \
    else { printf("  [FAIL] %s: want %.3f got %.3f\n", (msg), (double)(want), _v); g_fail++; } \
} while(0)

int c_main(void)
{
    puts("=== c_static_app: C -> smart_socket_ffi ===");
    puts("");

    puts("--- 1. round-trip ---");
    SmartSocket *s = socket_create("S1", 0, 1500.0);
    if (!s) { fputs("socket_create returned NULL\n", stderr); return 1; }
    print_state("after create:",   s);
    check("turn_on",  socket_turn_on(s));
    print_state("after turn_on:",  s);
    check("turn_off", socket_turn_off(s));
    print_state("after turn_off:", s);
    socket_destroy(s);
    puts("  destroy OK");
    putchar('\n');

    puts("--- 2. NULL safety ---");
    ASSERT_INT(socket_turn_on(NULL),  -1,   "turn_on(NULL)  == -1");
    ASSERT_INT(socket_turn_off(NULL), -1,   "turn_off(NULL) == -1");
    ASSERT_INT(socket_is_on(NULL),    -1,   "is_on(NULL)    == -1");
    ASSERT_DBL(socket_power(NULL), -1.0, 1e-9, "power(NULL) == -1.0");
    socket_destroy(NULL);
    puts("  destroy(NULL) OK");
    putchar('\n');

    puts("--- 3. initial state on ---");
    SmartSocket *s2 = socket_create("S2", 1, 800.0);
    ASSERT_INT(socket_is_on(s2),     1,       "is_on == 1");
    ASSERT_DBL(socket_power(s2), 800.0, 1e-6, "power == 800 W");
    socket_destroy(s2);
    putchar('\n');

    puts("--- 4. power == 0 when off ---");
    SmartSocket *s3 = socket_create("S3", 1, 600.0);
    check("turn_off", socket_turn_off(s3));
    ASSERT_INT(socket_is_on(s3),   0,     "is_on == 0");
    ASSERT_DBL(socket_power(s3), 0.0, 1e-9, "power == 0 W");
    socket_destroy(s3);
    putchar('\n');

    puts("--- 5. create with NULL name ---");
    SmartSocket *s4 = socket_create(NULL, 0, 100.0);
    ASSERT_INT(s4 == NULL ? 1 : 0, 1, "create(NULL) == NULL");
    socket_destroy(s4);
    putchar('\n');

    puts("--- 6. multiple instances ---");
    SmartSocket *a = socket_create("A", 0, 100.0);
    SmartSocket *b = socket_create("B", 0, 200.0);
    SmartSocket *c = socket_create("C", 0, 300.0);
    check("a.on", socket_turn_on(a));
    check("c.on", socket_turn_on(c));
    ASSERT_INT(socket_is_on(a), 1, "A on");
    ASSERT_INT(socket_is_on(b), 0, "B off");
    ASSERT_INT(socket_is_on(c), 1, "C on");
    ASSERT_DBL(socket_power(a), 100.0, 1e-6, "A power 100 W");
    ASSERT_DBL(socket_power(b),   0.0, 1e-9, "B power   0 W");
    ASSERT_DBL(socket_power(c), 300.0, 1e-6, "C power 300 W");
    socket_destroy(a); socket_destroy(b); socket_destroy(c);
    putchar('\n');

    puts("--- result ---");
    printf("  passed: %d\n", g_pass);
    printf("  failed: %d\n", g_fail);
    return g_fail > 0 ? 1 : 0;
}
