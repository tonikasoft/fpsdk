#include <Cocoa/Cocoa.h>

#include "add_child_window.h"
#include "wrapper.h"

void add_child_window(void *parent, void *child) {
    fplog("im in add_child_window");
    [(NSWindow *)parent addChildWindow:(NSWindow *)child ordered:NSWindowAbove];
}
