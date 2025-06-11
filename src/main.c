#include <stdio.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include <limits.h>
#include <gtk/gtk.h>
#include <webkit2/webkit2.h>
#include <version.h>
#include <notebook.h>
#include <libnotify/notify.h>
#include <plugin.h>

#define APP_ID              "com.polymath-studio.SimpleWebBrowser"
#define WIN_TITLE           "Simple Web Browser"
#define WIN_DEF_WIDTH       1280
#define WIN_DEF_HEIGHT      720
#define SPACING             5
#define PADDING             8
#define MICRO_BTN_WIDTH     16
#define MICRO_BTN_HEIGHT    16

static size_t N_PLUGINS_LOADED = 0;
static plugin_t *PLUGINS = NULL; // A collection of N_PLUGINS_LOADED dynamic functions
static GtkWidget *NOTEBOOK = NULL; // Reference to tab page. Use sparingly
static GtkWindow *WIN = NULL;
static GHashTable *DOWNLOADS = NULL; // Keep track of downloads so we don't loop forever

static void on_activate(GtkApplication *app, gpointer user_data);
static gboolean on_key_press(GtkWidget *widget, GdkEventKey *event, gpointer user_data);
static gboolean on_btn_press(GtkWidget *widget, GdkEventButton *event, gpointer user_data);
static void spawn_tab(GtkButton *btn, gpointer user_data);
static void close_tab(GtkButton *btn, gpointer user_data);
static void on_wv_title_changed(WebKitWebView *webview, GParamSpec *pspec, gpointer user_data);
static void on_wv_uri_changed(WebKitWebView *webview,  GParamSpec *pspec, gpointer user_data);
static void on_wv_download(WebKitWebContext *context, WebKitDownload *download, gpointer user_data);
static void on_wv_download_complete(WebKitDownload *download, gpointer user_data);
static void on_wv_download_failed(WebKitDownload *download, GError *error, gpointer user_data);
static WebKitWebView *on_wv_create(
    WebKitWebView *webview, WebKitNavigationAction *nav, gpointer user_data
);
static void on_tab_reordered(
    GtkNotebook *notebook, GtkWidget *child, guint page_num, gpointer user_data
);
static void on_tab_switched(
    GtkNotebook *notebook, GtkWidget *page, guint page_num, gpointer user_data
);
static void load_plugins(void);

int main(int argc, char **argv) {
    printf("%s v%u.%u by Dylan Turner\n", WIN_TITLE, MAJOR_VERS, MINOR_VERS);

    // Load dynamic libraries into the PLUGINS array
    load_plugins();

    // Start GUI
    setenv("WEBKIT_DISABLE_COMPOSITING_MODE", "1", 1); // Bc it won't work on nvidia otherwise
    GtkApplication *app = gtk_application_new(APP_ID, G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect(app, "activate", G_CALLBACK(on_activate), NULL);
    int status = g_application_run(G_APPLICATION(app), argc, argv);

    // Cleanup
    NOTEBOOK = NULL;
    g_object_unref(app);
    for (size_t i = 0; i < N_PLUGINS_LOADED; i++) {
        plugin__unload(PLUGINS + i);
    }
    free(PLUGINS);
    return status;
}

void notebook__spawn_tab(GtkNotebook *notebook, const char *const url) {
    GtkWidget *webview = webkit_web_view_new();
    webkit_web_view_load_uri(WEBKIT_WEB_VIEW(webview), url ? url : "https://search.brave.com/");

    GtkWidget *tab_lbl = gtk_label_new("New Tab");
    GtkWidget *tab_close_btn = gtk_button_new_with_label("x");
    gtk_button_set_relief(GTK_BUTTON(tab_close_btn), GTK_RELIEF_NONE);
    gtk_widget_set_size_request(tab_close_btn, MICRO_BTN_WIDTH, MICRO_BTN_HEIGHT);
    g_signal_connect(tab_close_btn, "clicked", G_CALLBACK(close_tab), webview);
    GtkWidget *tab_hbox = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, SPACING);
    gtk_box_pack_start(GTK_BOX(tab_hbox), tab_lbl, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(tab_hbox), tab_close_btn, FALSE, FALSE, 0);
    gtk_widget_show_all(tab_hbox);

    int n_pages = gtk_notebook_get_n_pages(GTK_NOTEBOOK(notebook));
    int pos = n_pages - 1;
    gtk_notebook_insert_page(GTK_NOTEBOOK(notebook), webview, tab_hbox, pos);
    gtk_notebook_set_tab_reorderable(GTK_NOTEBOOK(notebook), webview, TRUE);
    gtk_widget_show_all(GTK_WIDGET(notebook));
    gtk_notebook_set_current_page(GTK_NOTEBOOK(notebook), pos);

    // TODO: Tell plugins a new tab was created and selected
    // TODO: Set up call backs for this webview (including references to its tab position)
    g_signal_connect(webview, "notify::title", G_CALLBACK(on_wv_title_changed), tab_lbl);
    gtk_widget_add_events(webview, GDK_BUTTON_PRESS_MASK);
    g_signal_connect(webview, "button-press-event", G_CALLBACK(on_btn_press), NULL);
    g_signal_connect(webview, "notify::uri", G_CALLBACK(on_wv_uri_changed), NULL);
    g_signal_connect(webview, "create", G_CALLBACK(on_wv_create), NULL);

    WebKitWebContext *ctx = webkit_web_view_get_context(WEBKIT_WEB_VIEW(webview));
    g_signal_connect(ctx, "download-started", G_CALLBACK(on_wv_download), NULL);

    for (size_t i = 0; i < N_PLUGINS_LOADED; i++) {
        PLUGINS[i].on_new_tab(WEBKIT_WEB_VIEW(webview));
    }
}

// Build the UI in the window
static void on_activate(GtkApplication *app, gpointer user_data) {
    GtkWidget *win = gtk_application_window_new(app);
    WIN = GTK_WINDOW(win);
    gtk_window_set_title(GTK_WINDOW(win), WIN_TITLE);
    gtk_window_set_default_size(GTK_WINDOW(win), WIN_DEF_WIDTH, WIN_DEF_HEIGHT);
    GtkWidget *notebook = gtk_notebook_new();
    NOTEBOOK = notebook;

    DOWNLOADS = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, NULL);

    // New tab page
    GtkWidget *new_tab_btn = gtk_button_new_with_label("+");
    gtk_button_set_relief(GTK_BUTTON(new_tab_btn), GTK_RELIEF_NONE);
    gtk_widget_set_size_request(new_tab_btn, MICRO_BTN_WIDTH, MICRO_BTN_HEIGHT);
    g_signal_connect(new_tab_btn, "clicked", G_CALLBACK(spawn_tab), notebook);
    char lbl[1000] = "";
    snprintf(lbl, 1000, "Version: v%u.%u", MAJOR_VERS, MINOR_VERS);
    GtkWidget *lbl_app_name = gtk_label_new(WIN_TITLE);
    GtkWidget *lbl_created = gtk_label_new("by Dylan Turner");
    GtkWidget *lbl_vers = gtk_label_new(lbl);
    GtkWidget *new_tab_btn_content = gtk_box_new(GTK_ORIENTATION_VERTICAL, SPACING);
    gtk_box_pack_start(GTK_BOX(new_tab_btn_content), lbl_app_name, false, false, PADDING);
    gtk_box_pack_start(GTK_BOX(new_tab_btn_content), lbl_created, false, false, PADDING);
    gtk_box_pack_start(GTK_BOX(new_tab_btn_content), lbl_vers, false, false, PADDING);

    // The tab system
    gtk_notebook_append_page(GTK_NOTEBOOK(notebook), new_tab_btn_content, new_tab_btn);
    gtk_widget_set_margin_top(notebook, 0);
    gtk_widget_set_margin_bottom(notebook, PADDING);
    gtk_widget_set_margin_start(notebook, PADDING);
    gtk_widget_set_margin_end(notebook, PADDING);
    gtk_notebook_set_scrollable(GTK_NOTEBOOK(notebook), TRUE);
    g_signal_connect(notebook, "page-reordered", G_CALLBACK(on_tab_reordered), NULL);

    // The plugin bar
    GtkWidget *plugin_bar = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, SPACING);
    gtk_widget_set_margin_start(plugin_bar, PADDING);
    gtk_widget_set_margin_end(plugin_bar, PADDING);
    for (size_t i = 0; i < N_PLUGINS_LOADED; i++) {
        GtkWidget *plugin_widget = PLUGINS[i].create_bar_item(GTK_NOTEBOOK(notebook));
        if (plugin_widget == NULL) {
            continue; // Background plugin, like adblock
        }
        if (PLUGINS[i].is_pack_start()) {
            gtk_box_pack_start(
                GTK_BOX(plugin_bar), plugin_widget,
                PLUGINS[i].is_pack_expand(), PLUGINS[i].is_pack_fill(), 0
            );
        } else {
            gtk_box_pack_end(
                GTK_BOX(plugin_bar), plugin_widget,
                PLUGINS[i].is_pack_expand(), PLUGINS[i].is_pack_fill(), 0
            );
        }
    }
    gtk_widget_show_all(plugin_bar);

    // Put them together
    GtkWidget *outer_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_box_pack_start(GTK_BOX(outer_box), plugin_bar, FALSE, TRUE, PADDING);
    gtk_box_pack_start(GTK_BOX(outer_box), notebook, TRUE, TRUE, 0);
    gtk_container_add(GTK_CONTAINER(win), outer_box);

    // Final application signals
    g_signal_connect(win, "key-press-event", G_CALLBACK(on_key_press), NULL);
    g_signal_connect(notebook, "switch-page", G_CALLBACK(on_tab_switched), NULL);

    gtk_widget_show_all(win);
}

// Handle keyboard shortcuts
static gboolean on_key_press(GtkWidget *widget, GdkEventKey *event, gpointer user_data) {
    for (size_t i = 0; i < N_PLUGINS_LOADED; i++) {
        PLUGINS[i].on_key_press(event);
    }
    return FALSE;
}

// Handle button presses (like mouse buttons)
static gboolean on_btn_press(GtkWidget *widget, GdkEventButton *event, gpointer user_data) {
    for (size_t i = 0; i < N_PLUGINS_LOADED; i++) {
        PLUGINS[i].on_btn_press(event);
    }
    return FALSE;
}

// When the + tab is clicked, create a new page and put it at the end.
static void spawn_tab(GtkButton *btn, gpointer user_data) {
    notebook__spawn_tab(GTK_NOTEBOOK(user_data), NULL);
    for (size_t i = 0; i < N_PLUGINS_LOADED; i++) {
        PLUGINS[i].on_page_change();
    }
}

// When the x of a tab is clicked, remove it
static void close_tab(GtkButton *btn, gpointer user_data) {
    int page = gtk_notebook_page_num(GTK_NOTEBOOK(NOTEBOOK), GTK_WIDGET(user_data));
    int n_pages = gtk_notebook_get_n_pages(GTK_NOTEBOOK(NOTEBOOK));
    gtk_notebook_remove_page(GTK_NOTEBOOK(NOTEBOOK), page);
    gtk_widget_show_all(GTK_WIDGET(NOTEBOOK));
    if (page + 1 == n_pages - 1 && n_pages != 2) {
        n_pages = gtk_notebook_get_n_pages(GTK_NOTEBOOK(NOTEBOOK));
        gtk_notebook_set_current_page(GTK_NOTEBOOK(NOTEBOOK), n_pages - 2);
    }
}

// When the webpage loads its title, update the label
static void on_wv_title_changed(WebKitWebView *webview, GParamSpec *pspec, gpointer user_data) {
    GtkLabel *lbl = GTK_LABEL(user_data);
    const char *title = webkit_web_view_get_title(webview);
    if (title && *title) {
        gtk_label_set_text(lbl, title);
    } else {
        gtk_label_set_text(lbl, "Web Page Title");
    }
}

// When a webpage is loaded, we tell the plugins
static void on_wv_uri_changed(WebKitWebView *webview,  GParamSpec *pspec, gpointer user_data) {
    for (size_t i = 0; i < N_PLUGINS_LOADED; i++) {
        PLUGINS[i].on_page_change();
    }
}

// Handle downloads
static void on_wv_download(
        WebKitWebContext *context, WebKitDownload *download, gpointer user_data) {
    const gchar *uri = webkit_uri_request_get_uri(webkit_download_get_request(download));

    // Check if we've already restarted downloading this file
    if (g_hash_table_contains(DOWNLOADS, uri)) {
        g_hash_table_remove(DOWNLOADS, uri);
        return;
    }

    // Otherwise, cancel, and let user choose a save location
    webkit_download_cancel(download);
    g_hash_table_insert(DOWNLOADS, g_strdup(uri), GINT_TO_POINTER(1));
    GtkWidget *dialog = gtk_file_chooser_dialog_new(
        "Save File", WIN, GTK_FILE_CHOOSER_ACTION_SAVE,
        "_Cancel", GTK_RESPONSE_CANCEL,
        "_Save", GTK_RESPONSE_ACCEPT,
        NULL
    );
    gchar *filename = g_path_get_basename(uri);
    gtk_file_chooser_set_current_name(GTK_FILE_CHOOSER(dialog), filename);
    g_free(filename);
    if (gtk_dialog_run(GTK_DIALOG(dialog)) == GTK_RESPONSE_ACCEPT) {
        gchar *filepath = gtk_file_chooser_get_filename(GTK_FILE_CHOOSER(dialog));
        gchar *dest_uri = g_strdup_printf("file://%s", filepath);
        WebKitDownload *new_download = webkit_web_context_download_uri(context, uri);
        webkit_download_set_destination(new_download, dest_uri);
        g_signal_connect(new_download, "finished", G_CALLBACK(on_wv_download_complete), NULL);
        g_signal_connect(new_download, "failed", G_CALLBACK(on_wv_download_failed), NULL);
        g_free(dest_uri);
        g_free(filepath);
    }

    gtk_widget_destroy(dialog);
}

static void on_wv_download_complete(WebKitDownload *download, gpointer user_data) {
    const gchar *uri = webkit_uri_request_get_uri(webkit_download_get_request(download));
    g_hash_table_remove(DOWNLOADS, uri);
    const gchar *dest = webkit_download_get_destination(download);
    printf("[SWB] Download finished: %s\n", dest);
    notify_init("Download");
    NotifyNotification *n = notify_notification_new("Download Complete", dest, NULL);
    notify_notification_show(n, NULL);
    g_object_unref(n);
    notify_uninit();
}

static void on_wv_download_failed(WebKitDownload *download, GError *error, gpointer user_data) {
    // Ignore cancels
    if (g_error_matches(error, WEBKIT_DOWNLOAD_ERROR, WEBKIT_DOWNLOAD_ERROR_CANCELLED_BY_USER)) {
        return;
    }
    const gchar *uri = webkit_uri_request_get_uri(webkit_download_get_request(download));
    g_hash_table_remove(DOWNLOADS, uri);
    const gchar *dest = webkit_download_get_destination(download);
    printf("[SWB] Download failed: %s\n", dest);
    notify_init("Download Failed");
    NotifyNotification *n = notify_notification_new("Download Failed", dest, NULL);
    notify_notification_show(n, NULL);
    g_object_unref(n);
    notify_uninit();
}

// When user wants to "open in a new tab"
static WebKitWebView *on_wv_create(
        WebKitWebView *webview, WebKitNavigationAction *nav, gpointer user_data) {
    const gchar *uri = NULL;
    if (nav) {
        WebKitURIRequest *req = webkit_navigation_action_get_request(nav);
        if (req) {
            uri = webkit_uri_request_get_uri(req);
        }
    }
    notebook__spawn_tab(
        GTK_NOTEBOOK(NOTEBOOK), uri ? uri : "https://search.brave.com"
    );
    return NULL;
}

// Don't let the user move a tab past the plus button
static void on_tab_reordered(
        GtkNotebook *notebook, GtkWidget *child, guint page_num, gpointer user_data) {
    int n_pages = gtk_notebook_get_n_pages(notebook);
    int max_ind = n_pages - 2;
    if (page_num > max_ind) {
        gtk_notebook_reorder_child(notebook, child, max_ind);
    }
}

// Let plugins update their content when selecting a different tab
static void on_tab_switched(
        GtkNotebook *notebook, GtkWidget *page, guint page_num, gpointer user_data) {
    for (size_t i = 0; i < N_PLUGINS_LOADED; i++) {
        PLUGINS[i].on_tab_switched(page_num);
    }
}

// Load all plugins from various folders in order
static void load_plugins(void) {
    char **local_plugin_fnames = NULL;
    size_t n_local_plugins = 0;
    plugin__find_fnames(&local_plugin_fnames, &n_local_plugins, ".");
    char *plugin_folder = plugin__get_plugin_folder();
    char **cfg_plugin_fnames = NULL;
    size_t n_cfg_plugins = 0;
    plugin__find_fnames(&cfg_plugin_fnames, &n_cfg_plugins, plugin_folder);
    char **plugin_order = NULL;
    size_t n_plugins = 0;
    plugin__get_plugin_order(&plugin_order, &n_plugins);
    PLUGINS = malloc(sizeof(plugin_t) * n_plugins);
    N_PLUGINS_LOADED = 0;
    for (size_t i = 0; i < n_plugins; i++) {
        printf("Loading plugin '%s'\n", plugin_order[i]);
        bool in_cfg = false;
        for (size_t j = 0; j < n_cfg_plugins; j++) {
            if (strcmp(plugin_order[i], cfg_plugin_fnames[j]) == 0) {
                printf("Plugin %s is in config dir\n", plugin_order[i]);
                in_cfg = true;
                break;
            }
        }
        bool in_local = false;
        if (!in_cfg) {
            for (size_t j = 0; j < n_local_plugins; j++) {
                if (strcmp(plugin_order[i], local_plugin_fnames[j]) == 0) {
                    printf("Plugin %s is in local dir\n", plugin_order[i]);
                    in_local = true;
                    break;
                }
            }
        }
        if (!in_local && !in_cfg) {
            fprintf(stderr, "Warning! Couldn't find plugin '%s'\n", plugin_order[i]);
            continue;
        }
        char plugin_fname[PATH_MAX] = "";
        snprintf(
            plugin_fname, PATH_MAX,
            "%s/%s", in_local ? "." : plugin_folder, plugin_order[i]
        );
        plugin_t plugin = plugin__init(plugin_fname);
        if (plugin.handle == NULL) {
            fprintf(stderr, "Warning! Failed to load plugin '%s'\n", plugin_fname);
            continue;
        }
        int major_vers = plugin.on_load();
        if (major_vers != MAJOR_VERS) {
            fprintf(
                stderr, "Warning! Cannot load plugin '%s'. Made for different version.\n", plugin_fname
            );
            continue;
        }
        memcpy(PLUGINS + N_PLUGINS_LOADED, &plugin, sizeof(plugin_t));
        N_PLUGINS_LOADED++;
        printf("Loaded plugin '%s'\n", plugin_fname);
    }
    if (plugin_order != NULL) {
        for (size_t i = 0; i < n_plugins; i++) {
            free(plugin_order[i]);
        }
        free(plugin_order);
        plugin_order = NULL;
    }
    if (cfg_plugin_fnames != NULL) {
        for (size_t i = 0; i < n_cfg_plugins; i++) {
            free(cfg_plugin_fnames[i]);
        }
        free(cfg_plugin_fnames);
        cfg_plugin_fnames = NULL;
    }
    free(plugin_folder);
    if (local_plugin_fnames != NULL) {
        for (size_t i = 0; i < n_local_plugins; i++) {
            free(local_plugin_fnames[i]);
        }
        free(local_plugin_fnames);
        local_plugin_fnames = NULL;
    }
}
