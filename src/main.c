#include <stdio.h>
#include <gtk/gtk.h>
#include <webkit2/webkit2.h>

#define APP_ID              "com.polymath-studio.SimpleWebBrowser"
#define WIN_TITLE           "Simple Web Browser"
#define WIN_DEF_WIDTH       1280
#define WIN_DEF_HEIGHT      720
#define SPACING             5
#define PADDING             8
#define MICRO_BTN_WIDTH     16
#define MICRO_BTN_HEIGHT    16

static void on_activate(GtkApplication *app, gpointer user_data);
static void spawn_tab(GtkButton *btn, gpointer user_data);
static void close_tab(GtkButton *btn, gpointer user_data);
static void on_wv_title_changed(WebKitWebView *webview, GParamSpec *pspec, gpointer user_data);
static void on_tab_reordered(
    GtkNotebook *notebook, GtkWidget *child, guint page_num, gpointer user_data
);

static GtkWidget *NOTEBOOK = NULL; // Reference to tab page. Use sparingly

int main(int argc, char **argv) {
    setenv("WEBKIT_DISABLE_COMPOSITING_MODE", "1", 1); // Bc it won't work on nvidia otherwise
    GtkApplication *app = gtk_application_new(APP_ID, G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect(app, "activate", G_CALLBACK(on_activate), NULL);
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    return status;
}

// Build the UI in the window
static void on_activate(GtkApplication *app, gpointer user_data) {
    GtkWidget *win = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(win), WIN_TITLE);
    gtk_window_set_default_size(GTK_WINDOW(win), WIN_DEF_WIDTH, WIN_DEF_HEIGHT);

    // TODO: UI

    // The plugin bar
    GtkWidget *plugin_bar = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, SPACING);

    // The tab system
    GtkWidget *notebook = gtk_notebook_new();
    NOTEBOOK = notebook;
    GtkWidget *new_tab_btn = gtk_button_new_with_label("+");
    gtk_button_set_relief(GTK_BUTTON(new_tab_btn), GTK_RELIEF_NONE);
    gtk_widget_set_size_request(new_tab_btn, MICRO_BTN_WIDTH, MICRO_BTN_HEIGHT);
    g_signal_connect(new_tab_btn, "clicked", G_CALLBACK(spawn_tab), notebook);
    GtkWidget *new_tab_btn_content = gtk_label_new("");
    gtk_notebook_append_page(GTK_NOTEBOOK(notebook), new_tab_btn_content, new_tab_btn);
    gtk_widget_set_margin_top(notebook, PADDING);
    gtk_widget_set_margin_bottom(notebook, PADDING);
    gtk_widget_set_margin_start(notebook, PADDING);
    gtk_widget_set_margin_end(notebook, PADDING);
    gtk_notebook_set_scrollable(GTK_NOTEBOOK(notebook), TRUE);
    g_signal_connect(notebook, "page-reordered", G_CALLBACK(on_tab_reordered), NULL);

    GtkWidget *outer_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, SPACING);
    gtk_box_pack_start(GTK_BOX(outer_box), plugin_bar, FALSE, TRUE, PADDING);
    gtk_box_pack_start(GTK_BOX(outer_box), notebook, TRUE, TRUE, PADDING);
    gtk_container_add(GTK_CONTAINER(win), outer_box);

    gtk_widget_show_all(win);
}

static void spawn_tab(GtkButton *btn, gpointer user_data) {
    GtkWidget *webview = webkit_web_view_new();
    webkit_web_view_load_uri(WEBKIT_WEB_VIEW(webview), "https://search.brave.com/");

    GtkWidget *tab_lbl = gtk_label_new("New Tab");
    GtkWidget *tab_close_btn = gtk_button_new_with_label("x");
    gtk_button_set_relief(GTK_BUTTON(tab_close_btn), GTK_RELIEF_NONE);
    gtk_widget_set_size_request(tab_close_btn, MICRO_BTN_WIDTH, MICRO_BTN_HEIGHT);
    g_signal_connect(tab_close_btn, "clicked", G_CALLBACK(close_tab), webview);
    GtkWidget *tab_hbox = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, SPACING);
    gtk_box_pack_start(GTK_BOX(tab_hbox), tab_lbl, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(tab_hbox), tab_close_btn, FALSE, FALSE, 0);
    gtk_widget_show_all(tab_hbox);

    GtkNotebook *notebook = GTK_NOTEBOOK(user_data);
    int n_pages = gtk_notebook_get_n_pages(GTK_NOTEBOOK(notebook));
    int pos = n_pages - 1;
    gtk_notebook_insert_page(GTK_NOTEBOOK(notebook), webview, tab_hbox, pos);
    gtk_notebook_set_tab_reorderable(GTK_NOTEBOOK(notebook), webview, TRUE);
    gtk_widget_show_all(GTK_WIDGET(notebook));
    gtk_notebook_set_current_page(GTK_NOTEBOOK(notebook), pos);

    // TODO: Tell plugins a new tab was created and selected
    // TODO: Set up call backs for this webview (including references to its tab position)

    g_signal_connect(webview, "notify::title", G_CALLBACK(on_wv_title_changed), tab_lbl);
}

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

static void on_wv_title_changed(WebKitWebView *webview, GParamSpec *pspec, gpointer user_data) {
    GtkLabel *lbl = GTK_LABEL(user_data);
    const char *title = webkit_web_view_get_title(webview);
    if (title && *title) {
        gtk_label_set_text(lbl, title);
    } else {
        gtk_label_set_text(lbl, "Unknown");
    }
}

static void on_tab_reordered(
        GtkNotebook *notebook, GtkWidget *child, guint page_num, gpointer user_data) {
    int n_pages = gtk_notebook_get_n_pages(notebook);
    int max_ind = n_pages - 2;
    if (page_num > max_ind) {
        gtk_notebook_reorder_child(notebook, child, max_ind);
    }
}
