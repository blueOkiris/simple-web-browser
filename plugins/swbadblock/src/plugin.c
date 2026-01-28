// Add content filters to the webkit view

#include <stdio.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>
#include <gtk/gtk.h>
#include <curl/curl.h>
#include <webkit2/webkit2.h>
#include <json-c/json.h>
#include "../../../include/plugin.h"

#define MAJOR_VERS      1

// Structure to hold response data from curl
typedef struct {
    char *data;
    size_t size;
} resp_t;

// Array of filter list URLs to fetch and combine
static const char *FILTER_LIST_URLS[] = {
    "https://raw.githubusercontent.com/dgraham/Ka-Block/refs/heads/master/"
        "Extension/blockerList.json"
};

bool FILTER_LIST_REFRESHED = false;
static WebKitUserContentFilterStore *FILTER_STORE = NULL;
static GtkNotebook *NOTEBOOK = NULL; // Reference to the main content

static void on_filter_load(GObject *caller, GAsyncResult *res, void *conmgr);
static void on_filter_save(GObject *caller, GAsyncResult *res, void *conmgr);
static char *get_filter_list(void);
static char *fetch_url_to_string(const char *url);
static char *combine_json_arrays(void);

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


static size_t write_response_cb(void *contents, size_t size, size_t nmemb, resp_t *response) {
    size_t realsize = size * nmemb;
    char *ptr = realloc(response->data, response->size + realsize + 1);
    if (!ptr) {
        printf("[Swb Adblock] Error: Not enough memory (realloc returned NULL)\n");
        return 0;
    }
    response->data = ptr;
    memcpy(&(response->data[response->size]), contents, realsize);
    response->size += realsize;
    response->data[response->size] = 0;
    return realsize;
}

static char *fetch_url_to_string(const char *url) {
    CURL *curl = NULL;
    CURLcode res = { 0 };
    resp_t resp = { 0 };
    curl = curl_easy_init();
    if (!curl) {
        return NULL;
    }
    curl_easy_setopt(curl, CURLOPT_URL, url);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_response_cb);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &resp);
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);
    res = curl_easy_perform(curl);
    curl_easy_cleanup(curl);
    if (res != CURLE_OK) {
        printf("[Swb Adblock] Error fetching %s: %s\n", url, curl_easy_strerror(res));
        if (resp.data) {
            free(resp.data);
        }
        return NULL;
    }
    return resp.data;
}

static char *combine_json_arrays(void) {
    json_object *combined_array = json_object_new_array();
    if (!combined_array) {
        printf("[Swb Adblock] Error: Failed to create JSON array\n");
        return NULL;
    }
    size_t num_urls = sizeof(FILTER_LIST_URLS) / sizeof(FILTER_LIST_URLS[0]);
    for (size_t i = 0; i < num_urls; i++) {
        printf("[Swb Adblock] Fetching filter list from: %s\n", FILTER_LIST_URLS[i]);
        char *json_data = fetch_url_to_string(FILTER_LIST_URLS[i]);
        if (!json_data) {
            printf("[Swb Adblock] Warning: Failed to fetch %s\n", FILTER_LIST_URLS[i]);
            continue;
        }
        json_object *parsed_json = json_tokener_parse(json_data);
        free(json_data);
        if (!parsed_json) {
            printf("[Swb Adblock] Warning: Failed to parse JSON from %s\n", FILTER_LIST_URLS[i]);
            continue;
        }
        if (!json_object_is_type(parsed_json, json_type_array)) {
            printf("[Swb Adblock] Warning: JSON from %s is not an array\n", FILTER_LIST_URLS[i]);
            json_object_put(parsed_json);
            continue;
        }

        // Add all elements from this array to the combined array
        int array_len = json_object_array_length(parsed_json);
        for (int j = 0; j < array_len; j++) {
            json_object *element = json_object_array_get_idx(parsed_json, j);
            if (element) {
                // Increment reference count since we're adding it to another array
                json_object_get(element);
                json_object_array_add(combined_array, element);
            }
        }
        json_object_put(parsed_json);
        printf("[Swb Adblock] Added %d filters from %s\n", array_len, FILTER_LIST_URLS[i]);
    }

    const char *json_string = json_object_to_json_string(combined_array);
    char *result = strdup(json_string);
    if (result) {
        int total_filters = json_object_array_length(combined_array);
        printf("[Swb Adblock] Combined filter list created with %d total filters\n", total_filters);
    }
    json_object_put(combined_array);
    return result;
}

static char *get_filter_list(void) {
    return combine_json_arrays();
}
