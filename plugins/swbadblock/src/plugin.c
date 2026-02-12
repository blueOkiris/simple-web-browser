// Add content filters to the webkit view

#include <stdio.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>
#include <gtk/gtk.h>
#include <curl/curl.h>
#include <webkit2/webkit2.h>
#include <json-c/json.h>
#include <omp.h>
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
    ADGUARD_URL_BASE "AnnoyancesFilter/Cookies/sections/cookies_allowlist.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Cookies/sections/cookies_general.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Cookies/sections/cookies_specific.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/MobileApp/sections/mobile-app_allowlist.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/MobileApp/sections/mobile-app_general.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/MobileApp/sections/mobile-app_specific.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Other/sections/annoyances.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Other/sections/self-promo.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Other/sections/tweaks.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/antiadblock.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/popups_allowlist.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/popups_general.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/popups_specific.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/push-notifications_allowlist.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/push-notifications_general.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/push-notifications_specific.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/subscriptions_allowlist.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/subscriptions_general.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Popups/sections/subscriptions_specific.txt",
    ADGUARD_URL_BASE "AnnoyancesFilter/Widgets/sections/widgets.txt"
};

bool FILTER_LIST_REFRESHED = false;
static WebKitUserContentFilterStore *FILTER_STORE = NULL;
static GtkNotebook *NOTEBOOK = NULL; // Reference to the main content

typedef enum {
    FILTER_SAVE_PRIMARY = 0,
    FILTER_SAVE_FALLBACK_JSON_ONLY
} filter_save_mode_t;

typedef struct {
    WebKitUserContentManager *conmgr;
    GBytes *bytes;
    filter_save_mode_t mode;
} filter_save_ctx_t;

static void on_filter_load(GObject *caller, GAsyncResult *res, void *conmgr);
static void on_filter_save(GObject *caller, GAsyncResult *res, void *conmgr);
static void save_filter_list_async(
    WebKitUserContentManager *conmgr,
    char *filter_list,
    filter_save_mode_t mode
);
static char *get_filter_list(void);
static char *fetch_url_to_string(const char *url);
static char *combine_json_arrays(void);
static char *convert_adguard_rules_to_json(void);
static char *sanitize_adguard_rules(const char *rules);
static bool should_skip_adguard_rule(const char *line);

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
        if (filter_list == NULL) {
            printf("[Swb Adblock] Error! Failed to build filter list\n");
            return;
        }
        save_filter_list_async(WEBKIT_USER_CONTENT_MANAGER(conmgr), filter_list, FILTER_SAVE_PRIMARY);

        FILTER_LIST_REFRESHED = true;
    }
}

static void on_filter_save(GObject *caller, GAsyncResult *res, void *conmgr) {
    filter_save_ctx_t *ctx = (filter_save_ctx_t *)conmgr;
    GError *error = NULL;
    WebKitUserContentFilter *filter = webkit_user_content_filter_store_save_finish(
        FILTER_STORE, res, &error
    );

    if (error == NULL) {
        printf("[Swb Adblock] Saved new filter list\n");
        webkit_user_content_manager_add_filter(ctx->conmgr, filter);

        if (ctx->mode == FILTER_SAVE_FALLBACK_JSON_ONLY) {
            printf("[Swb Adblock] Applied fallback JSON-only blocklist\n");
        }
    } else {
        if (ctx->mode == FILTER_SAVE_PRIMARY) {
            printf("[Swb Adblock] Warning! Primary blocklist save failed: %s\n", error->message);

            char *fallback_filter_list = combine_json_arrays();
            if (fallback_filter_list != NULL) {
                printf("[Swb Adblock] Retrying with fallback JSON-only blocklist\n");
                save_filter_list_async(
                    ctx->conmgr,
                    fallback_filter_list,
                    FILTER_SAVE_FALLBACK_JSON_ONLY
                );
            } else {
                printf("[Swb Adblock] Error! Failed to build fallback JSON-only blocklist\n");
            }
        } else {
            printf("[Swb Adblock] Error! Failed to save fallback filter list: %s\n", error->message);
        }
        g_error_free(error);
    }

    if (ctx != NULL) {
        if (ctx->bytes != NULL) {
            g_bytes_unref(ctx->bytes);
        }
        if (ctx->conmgr != NULL) {
            g_object_unref(ctx->conmgr);
        }
        free(ctx);
    }
}

static void save_filter_list_async(
    WebKitUserContentManager *conmgr,
    char *filter_list,
    filter_save_mode_t mode
) {
    filter_save_ctx_t *ctx = calloc(1, sizeof(filter_save_ctx_t));
    if (ctx == NULL) {
        printf("[Swb Adblock] Error: Failed to allocate save context\n");
        free(filter_list);
        return;
    }

    ctx->conmgr = g_object_ref(conmgr);
    ctx->bytes = g_bytes_new_take((gpointer)filter_list, strlen(filter_list));
    ctx->mode = mode;

    webkit_user_content_filter_store_save(
        FILTER_STORE,
        "blocklist",
        ctx->bytes,
        NULL,
        on_filter_save,
        ctx
    );
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

    // Arrays to store results from parallel fetching
    char **json_data_array = calloc(num_urls, sizeof(char*));
    json_object **parsed_json_array = calloc(num_urls, sizeof(json_object*));
    if (!json_data_array || !parsed_json_array) {
        printf("[Swb Adblock] Error: Failed to allocate memory for parallel processing\n");
        free(json_data_array);
        free(parsed_json_array);
        json_object_put(combined_array);
        return NULL;
    }

    // Parallel fetch of all URLs
    #pragma omp parallel for schedule(dynamic)
    for (size_t i = 0; i < num_urls; i++) {
        json_data_array[i] = fetch_url_to_string(FILTER_LIST_URLS[i]);
        if (!json_data_array[i]) {
            printf("[Swb Adblock] Warning: Failed to fetch %s\n", FILTER_LIST_URLS[i]);
            continue;
        }
        parsed_json_array[i] = json_tokener_parse(json_data_array[i]);
        if (!parsed_json_array[i]) {
            printf("[Swb Adblock] Warning: Failed to parse JSON from %s\n", FILTER_LIST_URLS[i]);
            continue;
        }
        if (!json_object_is_type(parsed_json_array[i], json_type_array)) {
            printf("[Swb Adblock] Warning: JSON from %s is not an array\n", FILTER_LIST_URLS[i]);
            json_object_put(parsed_json_array[i]);
            parsed_json_array[i] = NULL;
            continue;
        }
    }

    // Sequential merge of results (must be sequential due to json-c thread safety)
    for (size_t i = 0; i < num_urls; i++) {
        if (parsed_json_array[i]) {
            int array_len = json_object_array_length(parsed_json_array[i]);
            for (int j = 0; j < array_len; j++) {
                json_object *element = json_object_array_get_idx(parsed_json_array[i], j);
                if (element) {
                    // Increment reference count since we're adding it to another array
                    json_object_get(element);
                    json_object_array_add(combined_array, element);
                }
            }
            json_object_put(parsed_json_array[i]);
        }
        if (json_data_array[i]) {
            free(json_data_array[i]);
        }
    }

    free(json_data_array);
    free(parsed_json_array);

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

    // Arrays to store results from parallel fetching and conversion
    char **rules_data_array = calloc(num_urls, sizeof(char*));
    char **webkit_json_array = calloc(num_urls, sizeof(char*));
    json_object **parsed_json_array = calloc(num_urls, sizeof(json_object*));
    if (!rules_data_array || !webkit_json_array || !parsed_json_array) {
        printf("[Swb Adblock] Error: Failed to allocate memory for parallel AdGuard processing\n");
        free(rules_data_array);
        free(webkit_json_array);
        free(parsed_json_array);
        json_object_put(combined_array);
        return NULL;
    }

    // Parallel fetch and conversion of all AdGuard URLs
    #pragma omp parallel for schedule(dynamic)
    for (size_t i = 0; i < num_urls; i++) {
        rules_data_array[i] = fetch_url_to_string(FILTER_LIST_URLS_ADGUARD[i]);
        if (!rules_data_array[i]) {
            printf(
                "[Swb Adblock] Warning: Failed to fetch AdGuard rules from %s\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
            continue;
        }

        // Remove modifiers/patterns that are known to generate invalid WebKit regex filters
        char *sanitized_rules = sanitize_adguard_rules(rules_data_array[i]);
        free(rules_data_array[i]);
        rules_data_array[i] = sanitized_rules;
        if (!rules_data_array[i]) {
            printf(
                "[Swb Adblock] Warning: Failed to sanitize AdGuard rules from %s\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
            continue;
        }

        // Convert AdGuard rules to WebKit JSON format
        webkit_json_array[i] = adguard2webkit__convert(rules_data_array[i]);
        if (!webkit_json_array[i]) {
            printf(
                "[Swb Adblock] Warning: Failed to convert AdGuard rules from %s\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
            continue;
        }

        parsed_json_array[i] = json_tokener_parse(webkit_json_array[i]);
        if (!parsed_json_array[i]) {
            printf(
                "[Swb Adblock] Warning: Failed to parse converted JSON from %s\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
            continue;
        }
        if (!json_object_is_type(parsed_json_array[i], json_type_array)) {
            printf(
                "[Swb Adblock] Warning: Converted JSON from %s is not an array\n",
                FILTER_LIST_URLS_ADGUARD[i]
            );
            json_object_put(parsed_json_array[i]);
            parsed_json_array[i] = NULL;
            continue;
        }
    }

    // Sequential merge of results (must be sequential due to json-c thread safety)
    for (size_t i = 0; i < num_urls; i++) {
        if (parsed_json_array[i]) {
            int array_len = json_object_array_length(parsed_json_array[i]);
            for (int j = 0; j < array_len; j++) {
                json_object *element = json_object_array_get_idx(parsed_json_array[i], j);
                if (element) {
                    // Increment reference count since we're adding it to another array
                    json_object_get(element);
                    json_object_array_add(combined_array, element);
                }
            }
            json_object_put(parsed_json_array[i]);
        }
        if (rules_data_array[i]) {
            free(rules_data_array[i]);
        }
        if (webkit_json_array[i]) {
            free(webkit_json_array[i]);
        }
    }
    free(rules_data_array);
    free(webkit_json_array);
    free(parsed_json_array);

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

static bool should_skip_adguard_rule(const char *line) {
    if (line == NULL || line[0] == '\0') {
        return true;
    }

    // Comments / section metadata
    if (line[0] == '!' || line[0] == '[') {
        return true;
    }

    // Cosmetic/HTML filtering rules not supported by content blockers
    if (
        strstr(line, "##") != NULL ||
        strstr(line, "#@#") != NULL ||
        strstr(line, "#?#") != NULL ||
        strstr(line, "#$#") != NULL ||
        strstr(line, "#%#") != NULL ||
        strstr(line, "#@$#") != NULL
    ) {
        return true;
    }

    // Drop raw regex rules (YARR frequently rejects these from upstream lists)
    const char *rule_start = line;
    if (rule_start[0] == '@' && rule_start[1] == '@') {
        rule_start += 2;
    }
    while (*rule_start == '|' || *rule_start == ' ' || *rule_start == '\t') {
        rule_start++;
    }
    if (rule_start[0] == '/') {
        return true;
    }

    // Drop modifiers that repeatedly produce unsupported rules in conversion
    if (
        strstr(line, "$replace") != NULL ||
        strstr(line, "$redirect") != NULL ||
        strstr(line, "$redirect-rule") != NULL ||
        strstr(line, "$jsonprune") != NULL ||
        strstr(line, "$removeparam") != NULL ||
        strstr(line, "$network") != NULL ||
        strstr(line, "$header") != NULL ||
        strstr(line, "$to=") != NULL ||
        strstr(line, "$method=") != NULL ||
        strstr(line, "$urlskip") != NULL ||
        strstr(line, "$urltransform") != NULL ||
        strstr(line, "$xmlprune") != NULL
    ) {
        return true;
    }

    return false;
}

static char *sanitize_adguard_rules(const char *rules) {
    if (rules == NULL) {
        return NULL;
    }

    size_t input_len = strlen(rules);
    char *out = calloc(input_len + 1, sizeof(char));
    if (out == NULL) {
        return NULL;
    }

    size_t out_len = 0;
    const char *line_start = rules;
    while (*line_start != '\0') {
        const char *line_end = line_start;
        while (*line_end != '\0' && *line_end != '\n' && *line_end != '\r') {
            line_end++;
        }

        // Trim surrounding whitespace
        const char *trim_start = line_start;
        while (trim_start < line_end && (*trim_start == ' ' || *trim_start == '\t')) {
            trim_start++;
        }
        const char *trim_end = line_end;
        while (trim_end > trim_start && (trim_end[-1] == ' ' || trim_end[-1] == '\t')) {
            trim_end--;
        }

        size_t line_len = (size_t)(trim_end - trim_start);
        if (line_len > 0) {
            char *line = strndup(trim_start, line_len);
            if (line == NULL) {
                free(out);
                return NULL;
            }

            if (!should_skip_adguard_rule(line)) {
                memcpy(out + out_len, line, line_len);
                out_len += line_len;
                out[out_len++] = '\n';
            }
            free(line);
        }

        while (*line_end == '\n' || *line_end == '\r') {
            line_end++;
        }
        line_start = line_end;
    }

    out[out_len] = '\0';
    return out;
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
    int adguard_len = json_object_array_length(adguard_array);
    int total_len = json_len + adguard_len;

    // Create temporary arrays to hold elements for parallel processing
    json_object **temp_elements = calloc(total_len, sizeof(json_object*));
    if (!temp_elements) {
        printf("[Swb Adblock] Error: Failed to allocate memory for parallel merging\n");
        json_object_put(json_array);
        json_object_put(adguard_array);
        json_object_put(combined_array);
        return NULL;
    }

    // Parallel extraction of elements from both arrays
    #pragma omp parallel sections
    {
        #pragma omp section
        {
            // Process JSON array elements
            for (int i = 0; i < json_len; i++) {
                json_object *element = json_object_array_get_idx(json_array, i);
                if (element) {
                    json_object_get(element);
                    temp_elements[i] = element;
                }
            }
        }
        #pragma omp section
        {
            // Process AdGuard array elements
            for (int i = 0; i < adguard_len; i++) {
                json_object *element = json_object_array_get_idx(adguard_array, i);
                if (element) {
                    json_object_get(element);
                    temp_elements[json_len + i] = element;
                }
            }
        }
    }

    // Sequential addition to combined array (json-c is not thread-safe for array modifications)
    for (int i = 0; i < total_len; i++) {
        if (temp_elements[i]) {
            json_object_array_add(combined_array, temp_elements[i]);
        }
    }
    free(temp_elements);
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
