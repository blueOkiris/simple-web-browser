// Implement loading plugins

#include <dirent.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <stddef.h>
#include <dlfcn.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <limits.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <errno.h>
#endif
#include <gtk/gtk.h>
#include <webkit2/webkit2.h>
#include <plugin.h>

static bool append_fnames(const char *const search_fldr, char ***dest, size_t *len);

char *plugin__get_plugin_folder(void) {
    char *buf = malloc(PATH_MAX);
    if (!buf) {
        return NULL;
    }
#ifdef _WIN32
    const char *dir = getenv("APPDATA");
    snprintf(buf, sizeof(buf), "%s/swb/plugins", dir);
    return buf;
#else
    const char *dir = getenv("XDG_CONFIG_HOME");
    if (!dir || !(*dir)) {
        const char *home = getenv("HOME");
        if (home && (*home)) {
            snprintf(buf, PATH_MAX, "%s/.config/swb/plugins", home);
            return buf;
        }
        free(buf);
        return NULL;
    } else {
        free(buf);
        return NULL;
    }
#endif
}

void plugin__find_fnames(char ***dest, size_t *len, const char *const folder) {
    if ((*dest) != NULL || len == NULL) {
        fprintf(stderr, "Warning! Improper call to plugin__find_fnames\n");
        return;
    }
    *len = 0;
    if (!append_fnames(folder, dest, len)) {
        return;
    }
}

void plugin__get_plugin_order(char ***dest, size_t *len) {
    if ((*dest) != NULL || len == NULL) {
        fprintf(stderr, "Warning! Improper call to plugin__get_plugin_order\n");
        return;
    }
    *len = 0;

    char plugins_fname[PATH_MAX] = "";
    char *plugin_folder = plugin__get_plugin_folder();
    snprintf(plugins_fname, PATH_MAX, "%s/plugins.txt", plugin_folder);
    free(plugin_folder);
    FILE *plugin_list = fopen(plugins_fname, "r");
    if (plugin_list) {
        // Override with ~/.config/swb/plugins/plugins.txt if it exists
        char line[PATH_MAX] = "";
        while (fgets(line, PATH_MAX, plugin_list)) {
            line[strcspn(line, "\n")] = '\0';
            printf("Will load plugin #%lu - %s\n", (*len) + 1, line);
            size_t linelen = strlen(line);
            if ((*dest) == NULL) {
                *dest = malloc(sizeof(char *));
                if ((*dest) == NULL) {
                    fprintf(stderr, "Warning! Cannot load plugins.txt. Out of memory.\n");
                    return;
                }
            } else {
                *dest = realloc((*dest), sizeof(char *) * (*len + 1));
            }
            (*dest)[*len] = malloc(sizeof(linelen) + 1);
            if ((*dest)[*len] == NULL) {
                fprintf(stderr, "Warning! Cannot load plugins.txt. Out of memory.\n");
                return;
            }
            strcpy((*dest)[*len], line);
            (*len)++;
        }
        fclose(plugin_list);
    } else {
        // Else, default to local
        snprintf(plugins_fname, PATH_MAX, "plugins.txt");
        plugin_list = fopen(plugins_fname, "r");
        if (!plugin_list) {
            fprintf(stderr, "Warning! Couldn't open plugins.txt\n");
            (*dest) = NULL;
            *len = 0;
            return;
        }
        char line[PATH_MAX] = "";
        while (fgets(line, PATH_MAX, plugin_list)) {
            line[strcspn(line, "\n")] = '\0';
            printf("Will load plugin #%lu - %s\n", (*len) + 1, line);
            size_t linelen = strlen(line);
            if ((*dest) == NULL) {
                *dest = malloc(sizeof(char *));
                if ((*dest) == NULL) {
                    fprintf(stderr, "Warning! Cannot load plugins.txt. Out of memory.\n");
                    return;
                }
            } else {
                *dest = realloc((*dest), sizeof(char *) * (*len + 1));
            }
            (*dest)[*len] = malloc(sizeof(linelen) + 1);
            if ((*dest)[*len] == NULL) {
                fprintf(stderr, "Warning! Cannot load plugins.txt. Out of memory.\n");
                return;
            }
            strcpy((*dest)[*len], line);
            (*len)++;
        }
        fclose(plugin_list);
    }
}

plugin_t plugin__init(char *fname) {
    plugin_t plugin = {
        NULL,
        NULL
    };
    plugin.handle = dlopen(fname, RTLD_NOW);
    if (!plugin.handle) {
        fprintf(stderr, "Warning: Dlopen failed for '%s': %s\n", fname, dlerror());
        return plugin;
    }

    fn_on_load on_load = (fn_on_load) dlsym(plugin.handle, "plugin__on_load");
    if (!on_load) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__on_load': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.on_load = on_load;

    fn_on_unload on_unload = (fn_on_unload) dlsym(plugin.handle, "plugin__on_unload");
    if (!on_unload) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__on_unload': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.on_unload = on_unload;

    fn_create_bar_item create_bar_item =
        (fn_create_bar_item) dlsym(plugin.handle, "plugin__create_bar_item");
    if (!create_bar_item) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__create_bar_item': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.create_bar_item = create_bar_item;

    fn_is_pack_start is_pack_start =
        (fn_is_pack_start) dlsym(plugin.handle, "plugin__is_pack_start");
    if (!is_pack_start) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__is_pack_start': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.is_pack_start = is_pack_start;

    fn_is_pack_expand is_pack_expand =
        (fn_is_pack_expand) dlsym(plugin.handle, "plugin__is_pack_expand");
    if (!is_pack_expand) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__is_pack_expand': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.is_pack_expand = is_pack_expand;

    fn_is_pack_fill is_pack_fill = (fn_is_pack_fill) dlsym(plugin.handle, "plugin__is_pack_fill");
    if (!is_pack_fill) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__is_pack_fill': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.is_pack_fill = is_pack_fill;

    fn_on_key_press on_key_press = (fn_on_key_press) dlsym(plugin.handle, "plugin__on_key_press");
    if (!on_key_press) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__on_key_press': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.on_key_press = on_key_press;

    fn_on_btn_press on_btn_press = (fn_on_btn_press) dlsym(plugin.handle, "plugin__on_btn_press");
    if (!on_btn_press) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__on_btn_press': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.on_btn_press = on_btn_press;

    fn_on_page_change on_page_change =
        (fn_on_page_change) dlsym(plugin.handle, "plugin__on_page_change");
    if (!on_page_change) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__on_page_change': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.on_page_change = on_page_change;

    fn_on_new_tab on_new_tab = (fn_on_new_tab) dlsym(plugin.handle, "plugin__on_new_tab");
    if (!on_new_tab) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__on_new_tab': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.on_new_tab = on_new_tab;

    fn_on_tab_switched on_tab_switched =
        (fn_on_tab_switched) dlsym(plugin.handle, "plugin__on_tab_switched");
    if (!on_tab_switched) {
        fprintf(
            stderr,
            "Warning! Dlsym failed for '%s' on 'plugin__on_tab_switched': %s\n",
            fname, dlerror()
        );
        dlclose(plugin.handle);
        plugin.handle = NULL;
        return plugin;
    }
    plugin.on_tab_switched = on_tab_switched;

    return plugin;
}

void plugin__unload(plugin_t *plugin) {
    plugin->on_unload();
    dlclose(plugin->handle);
}

static bool append_fnames(const char *const search_fldr, char ***dest, size_t *len) {
    DIR *dir = opendir(search_fldr);
    if (!dir) {
        fprintf(
            stderr,
            "Warning! Cannot load local plugins. Failed to open '%s' for reading\n",
            search_fldr
        );
        return false;
    }
    struct dirent *entry = NULL;
    while ((entry = readdir(dir))) {
        if (entry->d_type == DT_REG || entry->d_type == DT_LNK || entry->d_type == DT_UNKNOWN) {
            const char *name = entry->d_name;
            size_t namelen = strlen(name);
            if (namelen > 3 && strcmp(name + namelen - 3, ".so") == 0) {
                printf("Found plugin: %s/%s\n", search_fldr, name);
                if ((*dest) == NULL) {
                    *dest = malloc(sizeof(char *));
                    if ((*dest) == NULL) {
                        fprintf(stderr, "Warning! Cannot load plugins. Out of memory.\n");
                        return false;
                    }
                } else {
                    *dest = realloc(*dest, sizeof(char *) * (*len + 1));
                }
                (*dest)[*len] = malloc(sizeof(namelen) + 1);
                if ((*dest)[*len] == NULL) {
                    fprintf(stderr, "Warning! Cannot load plugins. Out of memory.\n");
                    return false;
                }
                strcpy((*dest)[*len], name);
                (*len)++;
            }
        }
    }
    closedir(dir);
    return true;
}

#ifndef _WIN32
static int mkdir_p(const char *path, mode_t mode) {
    char tmp[PATH_MAX] = "";
    snprintf(tmp, PATH_MAX, "%s", path);
    for (char *p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            mkdir(tmp, mode);  // ignore errors
            *p = '/';
        }
    }
    return mkdir(tmp, mode);  // final directory
}
#endif
