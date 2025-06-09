// Exposed notebook functions (mainly for plugin help)

#pragma once

#include <gtk/gtk.h>
#include <webkit2/webkit2.h>

void notebook__spawn_tab(GtkNotebook *notebook, const char *const url);
