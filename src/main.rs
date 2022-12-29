/*
 * Copyright (c) 2022 ttyS3
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use gtk::{
    prelude::*, ApplicationWindow, Inhibit, Notebook, Widget};

use webkit2gtk::{CacheModel, CookiePersistentStorage, UserContentManager, WebsiteDataManager};
use webkit2gtk::{WebContext, WebView};

use webkit2gtk::prelude::CookieManagerExt;
use webkit2gtk::prelude::WebContextExt;
use webkit2gtk::prelude::WebViewExt;
use webkit2gtk::prelude::WebkitSettingsExt;

use webkit2gtk::builders::{WebContextBuilder, WebViewBuilder, WebsiteDataManagerBuilder};

use glib::object::Cast;
// use gtk::prelude::GtkApplicationExt;
use gtk::prelude::GtkWindowExt;

use env_logger::{Builder, Target};
use gtk::Orientation;
use log::LevelFilter;
use std::path::Path;

fn main() {
    init_logger();

    gtk::init().unwrap();

    // let app = gtk::Application::builder().build();
    let app = gtk::Application::new(Some("com.github.gtk-rs.examples"), Default::default());

    app.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title(Some("webkit2gtk-rs-demo"));
        window.set_default_size(980, 700);

        let mut tabs = Vec::<gtk::Box>::new();
        let notebook = gtk::Notebook::new();

        let url_to_open = [
            "https://html5test.com",
            "https://bing.com",
            "https://github.com",
            "https://twitter.com",
            "https://youtube.com",
            "https://music.163.com",
        ];
        url_to_open.iter().for_each(|url| {
            create_tab_page(&notebook, url, &mut tabs);
        });

        notebook.show();
        // let v_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        // v_box.pack_start(&notebook.upcast(), false, false, 0);

        window.set_child(Some(&notebook.upcast::<Widget>()));

        /*let inspector = webview.get_inspector().unwrap();
        inspector.show();*/

        window.show();

        // ::delete-event signal has been remove in gtk4
        // see https://docs.gtk.org/gtk4/migrating-3to4.html#stop-using-gtkwidget-event-signals
        // If you were using ::delete-event to present a confirmation when using the close button of a window,
        // you should use the GtkWindow::close-request signal.
        // https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/prelude/trait.GtkWindowExt.html#tymethod.connect_close_request
        window.connect_close_request(|win| {
            log::info!("app exiting...");
            win.application().unwrap().quit();
            Inhibit(false)
        });
    });

    app.run();
}

fn create_tab_page(notebook: &Notebook, url: &str, tabs: &mut Vec<gtk::Box>) {
    // @TODO new_with_policies() unimplemented, we need AutoplayPolicy::Allow
    // https://webkitgtk.org/reference/webkit2gtk/stable/ctor.WebsitePolicies.new_with_policies.html
    let website_policies = webkit2gtk::WebsitePolicies::new();
    let web_context = create_web_context(Path::new("/tmp"));
    let ucm = UserContentManager::new();
    let webview = WebViewBuilder::new()
        .web_context(&web_context)
        .website_policies(&website_policies)
        .user_content_manager(&ucm)
        .build();
    webview.set_vexpand(true);
    webview.set_hexpand(true);
    webview.set_can_focus(true);
    init_webview_settings(false, &webview);

    // webview.settings().unwrap().set_enable_developer_extras(true);
    // multiple `settings` found
    // candidate #1 is defined in an impl of the trait `WebViewExt` for the type `O`
    // candidate #2 is defined in an impl of the trait `gtk::prelude::WidgetExt` for the type `O`
    // disambiguate the associated function for candidate #1
    // WebViewExt::settings(&webview).unwrap().set_enable_developer_extras(true);

    webview.load_uri(url);

    let tab = gtk::Box::new(Orientation::Horizontal, 0);
    let url_str = url.to_string();
    let title = url_str.trim_start_matches("https://");
    // the method `pack_start` exists for struct `gtk4::Box`, but its trait bounds were not satisfied
    tab.append(&gtk::Label::new(Some(title)).upcast::<Widget>());
    let index = notebook.append_page(&webview, Some(&tab));
    log::info!(
        "create_tab_page try add close button, title={} tab_index={}",
        title,
        index
    );
    // Standard Icon Names https://developer.gnome.org/icon-naming-spec/#names
    let button = gtk::Button::from_icon_name("window-close");
    button.set_has_frame(false);
    button.connect_clicked(glib::clone!(@weak notebook as notebook => move |_| {
        log::info!("close button click, tab_index={}", index);

        // webview.terminate_web_process();

        notebook.remove_page(Some(index));
    }));
    // prepend, append
    tab.append(&button);
    tab.show();
    tabs.push(tab);
}

fn init_webview_settings(forward_console_log: bool, webview: &WebView) {
    let settings = WebViewExt::settings(webview).unwrap();
    // enable developer console
    settings.set_enable_developer_extras(true);
    // forward web console message to Terminal
    if forward_console_log {
        settings.set_enable_write_console_messages_to_stdout(true);
        // Whether to draw compositing borders and repaint counters on layers drawn with accelerated compositing.
        // This is useful for debugging issues related to web content that is composited with the GPU.
        // see https://webkitgtk.org/reference/webkit2gtk/unstable/WebKitSettings.html#WebKitSettings--draw-compositing-indicators
        settings.set_draw_compositing_indicators(true);
    }

    // Enable webgl, webaudio, canvas features
    settings.set_enable_webgl(true);
    settings.set_enable_webaudio(true);
    // WebKitSettings:enable-accelerated-2d-canvas has been deprecated since version 2.32. and should not be used in newly-written code.
    // see https://webkitgtk.org/reference/webkit2gtk/unstable/WebKitSettings.html#WebKitSettings--enable-accelerated-2d-canvas
    // settings.set_enable_accelerated_2d_canvas(true);

    // disable Hardware Acceleration due to canvas `transform: translateZ(-1px)` style make text blurry
    settings.set_hardware_acceleration_policy(webkit2gtk::HardwareAccelerationPolicy::Always);

    // Enable App cache
    settings.set_enable_offline_web_application_cache(true);
    settings.set_enable_page_cache(true);

    settings.set_allow_universal_access_from_file_urls(true);
    settings.set_allow_file_access_from_file_urls(true);
    settings.set_allow_modal_dialogs(true);
    settings.set_allow_top_navigation_to_data_urls(true);

    settings.set_enable_html5_database(true);
    settings.set_enable_html5_local_storage(true);

    settings.set_enable_media(true);
    settings.set_enable_media_capabilities(true);
    settings.set_enable_media_stream(true);
    settings.set_enable_mediasource(true);
    settings.set_enable_mock_capture_devices(true);
    settings.set_enable_encrypted_media(true);
    settings.set_media_playback_allows_inline(true);
    settings.set_enable_webrtc(true);

    settings.set_enable_smooth_scrolling(true);
    settings.set_enable_javascript(true);
    settings.set_javascript_can_access_clipboard(true);
    settings.set_javascript_can_open_windows_automatically(true);

    // https://webkitgtk.org/reference/webkit2gtk/unstable/WebKitSettings.html#WebKitSettings--media-playback-requires-user-gesture
    // https://webkit.org/blog/7734/auto-play-policy-changes-for-macos/
    // fix js error:
    // Unhandled Promise Rejection: NotAllowedError: The request is not allowed by the user agent or the platform in the current context,
    // possibly because the user denied permission.
    settings.set_media_playback_requires_user_gesture(false);

    // font
    settings.set_default_charset("UTF-8");
    settings.set_default_font_family("Noto Sans CJK SC");
    settings.set_serif_font_family("Noto Serif CJK SC");
    settings.set_sans_serif_font_family("Noto Sans CJK SC");
}

fn create_web_context(base_dir: &Path) -> WebContext {
    let web_data_manager = new_web_data_manager(base_dir);

    let web_context = WebContextBuilder::new()
        .website_data_manager(&web_data_manager)
        .build();

    if let Some(cm) = web_context.cookie_manager() {
        CookieManagerExt::set_persistent_storage(
            &cm,
            base_dir.join("cookie").join("cookie.txt").to_str().unwrap(),
            CookiePersistentStorage::Text,
        );
    }
    log::info!("done initialize cookie persistent storage");

    web_context.set_cache_model(CacheModel::WebBrowser);

    web_context
}

fn new_web_data_manager(base_dir: &Path) -> WebsiteDataManager {
    // https://webkitgtk.org/reference/webkit2gtk/stable/class.WebsiteDataManager.html
    // websql_directory is deprecated since 2.24, WebSQL is no longer supported. Use IndexedDB instead.
    // see https://valadoc.org/webkit2gtk-4.0/WebKit.WebsiteDataManager.websql_directory.html
    let web_data_manager = WebsiteDataManagerBuilder::new()
        .base_cache_directory(base_dir.join("cache").to_str().unwrap())
        .base_data_directory(base_dir.join("data").to_str().unwrap())
        .disk_cache_directory(base_dir.join("disk").to_str().unwrap())
        .hsts_cache_directory(base_dir.join("hsts").to_str().unwrap())
        .indexeddb_directory(base_dir.join("db").to_str().unwrap())
        .local_storage_directory(base_dir.join("storage").to_str().unwrap())
        .offline_application_cache_directory(base_dir.join("offline").to_str().unwrap())
        .build();
    web_data_manager
}

pub fn init_logger() {
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/log.html
    Builder::new()
        .filter_level(LevelFilter::Debug) // set default level
        .parse_default_env() // then, if exists, respect the env config
        .target(Target::Stdout)
        .init();
}
