// API for loading plugins

#pragma once

#include <stddef.h>
#include <stdbool.h>
#include <gtk/gtk.h>
#include <webkit2/webkit2.h>

typedef int (*fn_on_load)(void);
typedef GtkWidget *(*fn_create_bar_item)(GtkNotebook *notebook);
typedef bool (*fn_is_pack_start)(void);
typedef bool (*fn_is_pack_expand)(void);
typedef bool (*fn_is_pack_fill)(void);
typedef void (*fn_on_key_press)(GdkEventKey *event);
typedef void (*fn_on_btn_press)(GdkEventButton *event);
typedef void (*fn_on_page_change)(void);

// Storage of the different functions a plugin will have
typedef struct {
    void *handle;
    fn_on_load on_load;
    fn_create_bar_item create_bar_item;
    fn_is_pack_start is_pack_start;
    fn_is_pack_expand is_pack_expand;
    fn_is_pack_fill is_pack_fill;
    fn_on_key_press on_key_press;
    fn_on_btn_press on_btn_press;
    fn_on_page_change on_page_change;
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
