#include "add_child_window.h"

void add_child_window(void *parent, void *child) {
    [(NSWindow *)parent addChildWindow:(NSWindow *)child ordered:NSWindowAbove];
}
