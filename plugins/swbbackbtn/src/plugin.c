// Implement back navigation button

#include <stdio.h>
#include <stdbool.h>
#include <gtk/gtk.h>
#include <webkit2/webkit2.h>

#define MAJOR_VERS      1

static void on_click(GtkButton *btn, gpointer user_data);
static void go_back(void);

static GtkNotebook *NOTEBOOK = NULL; // Reference to the main content

// When the plugin first gets loaded in. Return Major version supported
int plugin__on_load(void) {
    printf("[Swb Back Btn] I loaded successfully.\n");
    return MAJOR_VERS;
}

// What to put in the navigation bar
GtkWidget *plugin__create_bar_item(GtkNotebook *notebook) {
    NOTEBOOK = notebook;
    GtkWidget *btn = gtk_button_new_with_label("←");
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

// When a key is pressed. In this case, if Alt+Left is pressed, go back
void plugin__on_key_press(GdkEventKey *event) {
    if ((event->state & GDK_MOD1_MASK) && event->keyval == GDK_KEY_Left) {
        go_back();
    }
}

// When a btn is pressed. In this case, if mouse back, go back
void plugin__on_btn_press(GdkEventButton *event) {
    if (event->type == GDK_BUTTON_PRESS && event->button == 8) {
        go_back();
    }
}

// What to do when our button is clicked
static void on_click(GtkButton *btn, gpointer user_data) {
    go_back();
}

// Make the current webview go back
static void go_back(void) {
    // Get the current tab webview
    GtkWidget *current_page = gtk_notebook_get_nth_page(
        NOTEBOOK, gtk_notebook_get_current_page(NOTEBOOK)
    );

    if (!WEBKIT_IS_WEB_VIEW(current_page)) {
        return;
    }

    // Navigate back
    webkit_web_view_go_back(WEBKIT_WEB_VIEW(current_page));
}
