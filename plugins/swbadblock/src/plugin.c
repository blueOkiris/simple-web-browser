// Add content filters to the webkit view

#include <stdio.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>
#include <gtk/gtk.h>
#include <curl/curl.h>
#include <webkit2/webkit2.h>
#include "../../../include/plugin.h"

#define MAJOR_VERS      1

#define FILTER_LIST_URL \
    "https://raw.githubusercontent.com/dgraham/Ka-Block/refs/heads/master/Extension/" \
        "blockerList.json"

bool FILTER_LIST_REFRESHED = false;
static WebKitUserContentFilterStore *FILTER_STORE = NULL;
static GtkNotebook *NOTEBOOK = NULL; // Reference to the main content

static void on_filter_load(GObject *caller, GAsyncResult *res, void *conmgr);
static void on_filter_save(GObject *caller, GAsyncResult *res, void *conmgr);
static char *get_filter_list(void);

// When the plugin first gets loaded in. Return Major version supported
int plugin__on_load(void) {
    // Load the filter store
    char *plugins_dir = plugin__get_plugin_folder();
    size_t len = strlen(plugins_dir);
    plugins_dir = realloc(plugins_dir, len + strlen("/filter_store") + 1);
    strcpy(plugins_dir + len, "/filter_store");
    FILTER_STORE = webkit_user_content_filter_store_new(plugins_dir);
    free(plugins_dir);

    printf("[Swb Adblock] I loaded successfully.\n");
    return MAJOR_VERS;
}

// When plugin is deinitialized
void plugin__on_unload(void) {}

// What to put in the navigation bar (nothing, in this case)
GtkWidget *plugin__create_bar_item(GtkNotebook *notebook) {
    NOTEBOOK = notebook;
    return NULL;
}

// Whether to grow from the start or end of the plugin bar
bool plugin__is_pack_start(void) {
    return false;
}

// Should the box get extra available space
bool plugin__is_pack_expand(void) {
    return false;
}

// Should the box use all of the space given
bool plugin__is_pack_fill(void) {
    return false;
}

// When a key is pressed
void plugin__on_key_press(GdkEventKey *event) {}

// When a btn is pressed
void plugin__on_btn_press(GdkEventButton *event) {}

// When the webview changes pages
void plugin__on_page_change(void) {}

// When a new tab is created
void plugin__on_new_tab(WebKitWebView *webview) {
    WebKitUserContentManager *conmgr = webkit_web_view_get_user_content_manager(webview);
    webkit_user_content_filter_store_load(FILTER_STORE, "blocklist", NULL, on_filter_load, conmgr);
}

// When a tab is clicked on
void plugin__on_tab_switched(guint page) {}

// Once a filter store has been loaded into a webview
static void on_filter_load(GObject *caller, GAsyncResult *res, void *conmgr) {
    GError *error = NULL;
    WebKitUserContentFilter *filter = webkit_user_content_filter_store_load_finish(
        FILTER_STORE, res, &error
    );
    if (error == NULL && FILTER_LIST_REFRESHED) {
        printf("[Swb Adblock] Cached content filter applied successfully\n");
        webkit_user_content_manager_add_filter(WEBKIT_USER_CONTENT_MANAGER(conmgr), filter);
    } else {
        // Haven't saved the filter list before
        if (error != NULL) {
            fprintf(
                stderr,
                "[Swb Adblock] Warning! %s\n", error->message
            );
        }

        char *filter_list = get_filter_list();
        GBytes *bytes = g_bytes_new_static(filter_list, strlen(filter_list));
        webkit_user_content_filter_store_save(
            FILTER_STORE, "blocklist", bytes, NULL, on_filter_save, conmgr
        );
        g_bytes_unref(bytes);
        free(filter_list);

        FILTER_LIST_REFRESHED = true;
    }
}

static void on_filter_save(GObject *caller, GAsyncResult *res, void *conmgr) {
    GError *error = NULL;
    WebKitUserContentFilter *filter = webkit_user_content_filter_store_save_finish(
        FILTER_STORE, res, &error
    );
    if (error == NULL) {
        printf("[Swb Adblock] Saved new filter list\n");
        webkit_user_content_manager_add_filter(WEBKIT_USER_CONTENT_MANAGER(conmgr), filter);
    } else {
        printf("[Swb Adblock] Error! Failed to save filter list: %s\n", error->message);
    }
}

static size_t write_data(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    return fwrite(ptr, size, nmemb, stream);
}

static char *get_filter_list(void) {
    CURL *curl = curl_easy_init();
    if (curl) {
        FILE *fp = fopen("/tmp/filter_list.json", "wb");
        if (!fp) {
            return NULL;
        }
        curl_easy_setopt(curl, CURLOPT_URL, FILTER_LIST_URL);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_data);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, fp);
        curl_easy_perform(curl);
        curl_easy_cleanup(curl);
        fclose(fp);
        fp = fopen("/tmp/filter_list.json", "r");
        if (!fp) {
            return NULL;
        }
        fseek(fp, 0, SEEK_END);
        size_t size = ftell(fp);
        rewind(fp);
        char *buff = malloc(size + 1);
        if (!buff) {
            fclose(fp);
            return NULL;
        }
        fread(buff, 1, size, fp);
        fclose(fp);
        buff[size] = '\0';
        return buff;
    }
    return NULL;
}
