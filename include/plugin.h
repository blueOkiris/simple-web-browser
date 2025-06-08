// API for loading plugins

#pragma once

#include <stddef.h>

typedef void (*fn_on_load)(void);

// Storage of the different functions a plugin will have
typedef struct {
    void *handle;
    fn_on_load on_load;
} plugin_t;

// Find ~/.config/swb/plugins/ (or equivalent)
char *plugin__get_plugin_folder(void);

// Load a list of plugin names. Must be freed
void plugin__find_fnames(char ***dest, size_t *len, const char *const folder);

// Load a corresponding list of plugin names that give the order
void plugin__get_plugin_order(char ***dest, size_t *len);

// Start a plugin and load it into memory
plugin_t plugin__init(char *fname);

// Clean up a plugin
void plugin__unload(plugin_t *plugin);
