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
#include "../adguard2webkit/include/adguard2webkit.h"

#define MAJOR_VERS          1
#define ADGUARD_URL_BASE    \
    "https://raw.githubusercontent.com/AdguardTeam/AdguardFilters/refs/heads/master/"

// Structure to hold response data from curl
typedef struct {
    char *data;
    size_t size;
} resp_t;

// Array of filter list URLs to fetch and combine
static const char *FILTER_LIST_URLS[] = {
    "https://raw.githubusercontent.com/dgraham/Ka-Block/refs/heads/master/"
        "Extension/blockerList.json",
    "https://raw.githubusercontent.com/AdguardTeam/AdguardForiOS/refs/heads/master/"
        "AdguardExtension/YouTubeAdsActionExtension/filter.json"
};
static const char *FILTER_LIST_URLS_ADGUARD[] = {
    // AdGuard Base Filter
    ADGUARD_URL_BASE "BaseFilter/sections/adservers.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/adservers_firstparty.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/allowlist.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/allowlist_stealth.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/antiadblock.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/banner_sizes.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/content_blocker.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/cryptominers.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/foreign.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/general_elemhide.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/general_extensions.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/general_url.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/replace.txt",
    ADGUARD_URL_BASE "BaseFilter/sections/specific.txt",

    // AdGuard Annoyances Filter
    ADGUARD_URL_BASE "AnnoyancesFilter/Cookies/sections/cookies_allowlist.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Cookies/sections/cookies_general.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Cookies/sections/cookies_specific.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/MobileApp/sections/mobile-app_allowlist.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/MobileApp/sections/mobile-app_general.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/MobileApp/sections/mobile-app_specific.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Other/sections/annoyances.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Other/sections/self-promo.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Other/sections/tweaks.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/antiadblock.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/popups_allowlist.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/popups_general.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/popups_specific.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/push-notifications_allowlist.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/push-notifications_general.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/push-notifications_specific.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/subscriptions_allowlist.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/subscriptions_general.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/subscriptions_specific.txt"
    ADGUARD_URL_BASE "AnnoyancesFilter/Widgets/sections/widgets.txt"
};

bool FILTER_LIST_REFRESHED = false;
static WebKitUserContentFilterStore *FILTER_STORE = NULL;
static GtkNotebook *NOTEBOOK = NULL; // Reference to the main content

static void on_filter_load(GObject *caller, GAsyncResult *res, void *conmgr);
static void on_filter_save(GObject *caller, GAsyncResult *res, void *conmgr);
static char *get_filter_list(void);
static char *fetch_url_to_string(const char *url);
static char *combine_json_arrays(void);
static char *convert_adguard_rules_to_json(void);

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

static char *convert_adguard_rules_to_json(void) {
    json_object *combined_array = json_object_new_array();
    if (!combined_array) {
        printf("[Swb Adblock] Error: Failed to create JSON array for AdGuard rules\n");
        return NULL;
    }
    size_t num_urls = sizeof(FILTER_LIST_URLS_ADGUARD) / sizeof(FILTER_LIST_URLS_ADGUARD[0]);
    for (size_t i = 0; i < num_urls; i++) {
        printf("[Swb Adblock] Fetching AdGuard rules from: %s\n", FILTER_LIST_URLS_ADGUARD[i]);
        char *rules_data = fetch_url_to_string(FILTER_LIST_URLS_ADGUARD[i]);
        if (!rules_data) {
            printf(
                "[Swb Adblock] Warning: Failed to fetch AdGuard rules from %s\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
            continue;
        }

        // Convert AdGuard rules to WebKit JSON format
        char *webkit_json = adguard2webkit__convert(rules_data);
        free(rules_data);
        if (!webkit_json) {
            printf(
                "[Swb Adblock] Warning: Failed to convert AdGuard rules from %s\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
            continue;
        }

        json_object *parsed_json = json_tokener_parse(webkit_json);
        free(webkit_json);
        if (!parsed_json) {
            printf(
                "[Swb Adblock] Warning: Failed to parse converted JSON from %s\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
            continue;
        }
        if (!json_object_is_type(parsed_json, json_type_array)) {
            printf(
                "[Swb Adblock] Warning: Converted JSON from %s is not an array\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
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
        printf(
            "[Swb Adblock] Added %d converted AdGuard filters from %s\n",
            array_len, FILTER_LIST_URLS_ADGUARD[i]
        );
    }

    const char *json_string = json_object_to_json_string(combined_array);
    char *result = strdup(json_string);
    if (result) {
        int total_filters = json_object_array_length(combined_array);
        printf(
            "[Swb Adblock] Converted AdGuard filter list created with %d total filters\n",
            total_filters
        );
    }
    json_object_put(combined_array);
    return result;
}

static char *get_filter_list(void) {
    // Get existing JSON filters
    char *json_filters = combine_json_arrays();
    if (!json_filters) {
        printf("[Swb Adblock] Warning: Failed to get JSON filters, using only AdGuard rules\n");
        return convert_adguard_rules_to_json();
    }

    // Get converted AdGuard rules
    char *adguard_filters = convert_adguard_rules_to_json();
    if (!adguard_filters) {
        printf("[Swb Adblock] Warning: Failed to convert AdGuard rules, using only JSON filters\n");
        return json_filters;
    }

    // Parse both JSON arrays
    json_object *json_array = json_tokener_parse(json_filters);
    json_object *adguard_array = json_tokener_parse(adguard_filters);
    free(json_filters);
    free(adguard_filters);
    if (!json_array || !adguard_array) {
        printf("[Swb Adblock] Error: Failed to parse filter arrays for combination\n");
        if (json_array) json_object_put(json_array);
        if (adguard_array) json_object_put(adguard_array);
        return NULL;
    }

    // Add JSON filters to combined array
    json_object *combined_array = json_object_new_array();
    if (!combined_array) {
        printf("[Swb Adblock] Error: Failed to create combined JSON array\n");
        json_object_put(json_array);
        json_object_put(adguard_array);
        return NULL;
    }
    int json_len = json_object_array_length(json_array);
    for (int i = 0; i < json_len; i++) {
        json_object *element = json_object_array_get_idx(json_array, i);
        if (element) {
            json_object_get(element);
            json_object_array_add(combined_array, element);
        }
    }
    int adguard_len = json_object_array_length(adguard_array);
    for (int i = 0; i < adguard_len; i++) {
        json_object *element = json_object_array_get_idx(adguard_array, i);
        if (element) {
            json_object_get(element);
            json_object_array_add(combined_array, element);
        }
    }
    json_object_put(json_array);
    json_object_put(adguard_array);

    const char *json_string = json_object_to_json_string(combined_array);
    char *result = strdup(json_string);
    if (result) {
        int total_filters = json_object_array_length(combined_array);
        printf(
            "[Swb Adblock] Final combined filter list created with %d total filters "
            "(%d JSON + %d AdGuard)\n",
            total_filters, json_len, adguard_len
        );
    }
    json_object_put(combined_array);
    return result;
}
