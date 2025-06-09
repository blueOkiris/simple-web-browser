// Implement forward navigation button

#include <stdio.h>
#include <stdbool.h>
#include <gtk/gtk.h>
#include <webkit2/webkit2.h>

#define MAJOR_VERS      1

static GtkNotebook *NOTEBOOK = NULL; // Reference to the main content

static void on_click(GtkButton *btn, gpointer user_data);
static void go_fwd(void);

// When the plugin first gets loaded in
int plugin__on_load(void) {
    printf("[Swb Fwd Btn] I loaded successfully.\n");
    return MAJOR_VERS;
}

// When plugin is deinitialized
void plugin__on_unload(void) {}

// What to put in the navigation bar
GtkWidget *plugin__create_bar_item(GtkNotebook *notebook) {
    NOTEBOOK = notebook;
    GtkWidget *btn = gtk_button_new_with_label("→");
    g_signal_connect(btn, "clicked", G_CALLBACK(on_click), NULL);
    return btn;
}

// Whether to grow from the start or end of the plugin bar
bool plugin__is_pack_start(void) {
    return true;
}

// Should the box get extra available space
bool plugin__is_pack_expand(void) {
    return false;
}

// Should the box use all of the space given
bool plugin__is_pack_fill(void) {
    return false;
}

// When a key is pressed. In this case, if Alt+Right is pressed, go forward
void plugin__on_key_press(GdkEventKey *event) {
    if ((event->state & GDK_MOD1_MASK) && event->keyval == GDK_KEY_Right) {
        go_fwd();
    }
}

// When a btn is pressed. In this case, if mouse fwd, go fwd
void plugin__on_btn_press(GdkEventButton *event) {
    if (event->type == GDK_BUTTON_PRESS && event->button == 9) {
        go_fwd();
    }
}

// When the webview changes pages
void plugin__on_page_change(void) {}

// When a new tab is created
void plugin__on_new_tab(WebKitWebView *webview) {}

// When a tab is clicked on
void plugin__on_tab_switched(guint page) {}

// What to do when our button is clicked
static void on_click(GtkButton *btn, gpointer user_data) {
    go_fwd();
}

// Make the current webview go forward
static void go_fwd(void) {
    // Get the current tab webview
    GtkWidget *current_page = gtk_notebook_get_nth_page(
        NOTEBOOK, gtk_notebook_get_current_page(NOTEBOOK)
    );

    if (!WEBKIT_IS_WEB_VIEW(current_page)) {
        return;
    }

    // Navigate back
    webkit_web_view_go_forward(WEBKIT_WEB_VIEW(current_page));
}
