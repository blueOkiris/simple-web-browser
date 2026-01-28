// Implement a bar for searching/going to websites

#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <gtk/gtk.h>
#include <webkit2/webkit2.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <limits.h>
#endif
#include "../../../include/notebook.h"

#define MAJOR_VERS          1

#define DEF_SEARCH_ENGINE   "https://search.brave.com/search?q=%s"

static GtkNotebook *NOTEBOOK = NULL; // Reference to the main content
static GtkEntry *ENTRY = NULL; // Reference to our drawn data
static char *CUSTOM_SEARCH_ENGINE = NULL; // Allow users to use their own search query string

static void on_entry_activate(GtkEntry *entry, gpointer user_data);
static void load_custom_search_engine(void);

// When the plugin first gets loaded in
int plugin__on_load(void) {
    load_custom_search_engine();
    printf("[Swb Navbar] I loaded successfully.\n");
    return MAJOR_VERS;
}

// When plugin is deinitialized
void plugin__on_unload(void) {
    if (CUSTOM_SEARCH_ENGINE != NULL) {
        free(CUSTOM_SEARCH_ENGINE);
    }
}

// What to put in the navigation bar
GtkWidget *plugin__create_bar_item(GtkNotebook *notebook) {
    NOTEBOOK = notebook;
    GtkWidget *entry = gtk_entry_new();
    ENTRY = GTK_ENTRY(entry);
    g_signal_connect(entry, "activate", G_CALLBACK(on_entry_activate), NULL);
    return entry;
}

// Whether to grow from the start or end of the plugin bar
bool plugin__is_pack_start(void) {
    return true;
}

// Should the box get extra available space
bool plugin__is_pack_expand(void) {
    return true;
}

// Should the box use all of the space given
bool plugin__is_pack_fill(void) {
    return true;
}

// When a key is pressed
void plugin__on_key_press(GdkEventKey *event) {}

// When a btn is pressed
void plugin__on_btn_press(GdkEventButton *event) {}

// When the webview changes pages
void plugin__on_page_change(void) {
    // Get the current page uri
    GtkWidget *current_page = gtk_notebook_get_nth_page(
        NOTEBOOK, gtk_notebook_get_current_page(NOTEBOOK)
    );
    if (!WEBKIT_IS_WEB_VIEW(current_page)) {
        return;
    }
    const gchar *uri = webkit_web_view_get_uri(WEBKIT_WEB_VIEW(current_page));

    // Update text
    gtk_entry_set_text(ENTRY, uri);
    gtk_widget_show_all(GTK_WIDGET(ENTRY));
}

// When a new tab is created
void plugin__on_new_tab(WebKitWebView *webview) {}

// When a tab is clicked on
void plugin__on_tab_switched(guint page) {
    // Get the current page uri
    GtkWidget *page_content = gtk_notebook_get_nth_page(NOTEBOOK, page);
    if (!WEBKIT_IS_WEB_VIEW(page_content)) {
        return;
    }
    const gchar *uri = webkit_web_view_get_uri(WEBKIT_WEB_VIEW(page_content));

    // Update text
    gtk_entry_set_text(ENTRY, uri);
    gtk_widget_show_all(GTK_WIDGET(ENTRY));
}

// Figure out if the text is a url or not and format it if so
static void format_url(char *buff, size_t buff_max, const char *const text) {
    if (strchr(text, '.')) {
        // Treat as a url
        if (strncmp(text, "https://", 8) != 0) {
            snprintf(buff, buff_max, "https://%s", text);
        } else {
            strcpy(buff, text);
        }
    } else {
        // Treat as search query
        snprintf(
            buff, buff_max,
            CUSTOM_SEARCH_ENGINE != NULL ? CUSTOM_SEARCH_ENGINE : DEF_SEARCH_ENGINE, text
        );
    }
    char buff2[buff_max];
    size_t len = strlen(buff);
    for (int i = 0; i < len; i++) {
        char entry[4] = "";
        switch (buff[i]) {
            case ' ':
                strcpy(entry, "%20");
                break;
            case '+':
                strcpy(entry, "%2B");
                break;
            case '#':
                strcpy(entry, "%23");
                break;
        }
        if (strlen(entry) > 0) {
            strcpy(buff2, buff);
            buff2[i] = 0;
            char *second_half = buff2 + i + 1;
            snprintf(buff, buff_max, "%s%s%s", buff2, entry, second_half);
            len += 4;
        }
    }
}

// Load custom search engine from config file
static void load_custom_search_engine(void) {
    // Get the config directory path
    char *config_dir = NULL;
#ifdef _WIN32
    const char *appdata = getenv("APPDATA");
    if (appdata && *appdata) {
        size_t len = strlen(appdata) + strlen("/swb") + 1;
        config_dir = malloc(len);
        if (config_dir) {
            snprintf(config_dir, len, "%s/swb", appdata);
        }
    }
#else
    const char *xdg_config = getenv("XDG_CONFIG_HOME");
    if (xdg_config && *xdg_config) {
        size_t len = strlen(xdg_config) + strlen("/swb") + 1;
        config_dir = malloc(len);
        if (config_dir) {
            snprintf(config_dir, len, "%s/swb", xdg_config);
        }
    } else {
        const char *home = getenv("HOME");
        if (home && *home) {
            size_t len = strlen(home) + strlen("/.config/swb") + 1;
            config_dir = malloc(len);
            if (config_dir) {
                snprintf(config_dir, len, "%s/.config/swb", home);
            }
        }
    }
#endif
    if (!config_dir) {
        return; // Could not determine config directory
    }

    // Build the path to .search_engine file
    size_t path_len = strlen(config_dir) + strlen("/.search_engine") + 1;
    char *search_engine_path = malloc(path_len);
    if (!search_engine_path) {
        free(config_dir);
        return;
    }
    snprintf(search_engine_path, path_len, "%s/.search_engine", config_dir);
    free(config_dir);

    // Try to open and read the file
    FILE *file = fopen(search_engine_path, "r");
    free(search_engine_path);
    if (!file) {
        return; // File doesn't exist, use default search engine
    }

    // Read the first line
    char line[1024];
    if (fgets(line, sizeof(line), file)) {
        // Remove trailing newline if present
        line[strcspn(line, "\n")] = '\0';

        // Only use non-empty lines
        if (strlen(line) > 0) {
            // Free existing custom search engine if any
            if (CUSTOM_SEARCH_ENGINE) {
                free(CUSTOM_SEARCH_ENGINE);
            }

            // Allocate and copy the custom search engine
            CUSTOM_SEARCH_ENGINE = malloc(strlen(line) + 1);
            if (CUSTOM_SEARCH_ENGINE) {
                strcpy(CUSTOM_SEARCH_ENGINE, line);
                printf("[Swb Navbar] Loaded custom search engine: %s\n", CUSTOM_SEARCH_ENGINE);
            }
        }
    }

    fclose(file);
}

// When the user presses enter
static void on_entry_activate(GtkEntry *entry, gpointer user_data) {
    const gchar *text = gtk_entry_get_text(entry);
    char url[1000] = "";
    format_url(url, 1000, text);
    gtk_entry_set_text(entry, url);
    gtk_widget_show_all(GTK_WIDGET(entry));

    // Get the current tab webview
    GtkWidget *current_page = gtk_notebook_get_nth_page(
        NOTEBOOK, gtk_notebook_get_current_page(NOTEBOOK)
    );

    if (!WEBKIT_IS_WEB_VIEW(current_page)) {
        notebook__spawn_tab(NOTEBOOK, url);
    } else {
        webkit_web_view_load_uri(WEBKIT_WEB_VIEW(current_page), url);
    }
}
