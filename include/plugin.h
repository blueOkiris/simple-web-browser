// API for loading plugins

#pragma once

#include <stddef.h>

// Find ~/.config/swb/plugins/ (or equivalent)
char *plugin__get_plugin_folder(void);

// Load a list of plugin names. Must be freed
void plugin__find_fnames(char **dest, size_t *len, const char *const folder);

// Load a corresponding list of plugin names that give the order
void plugin__get_plugin_order(char **dest, size_t *len);
