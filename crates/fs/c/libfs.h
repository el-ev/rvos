#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


void __panic(void);

void set_panic_display(void (*f)(const char*));
