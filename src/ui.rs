use gtk4::{gdk, prelude::*};

use gio::glib::BoolError;

fn activate_css() {
    let css_provider = gtk4::CssProvider::new();
    let css_src = include_str!("gtk/style.css");
    css_provider.load_from_string(css_src);
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Couldn't connect to default display."),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn on_activate(application: &gtk4::Application) {
    activate_css();
    let src = include_str!("gtk/layout.ui");
    let builder = gtk4::Builder::from_string(src);
    let window = builder
        .object::<gtk4::ApplicationWindow>("window")
        .expect("Unable to parse GTK root element.");
    window.add_css_class("testbox");
    window.set_application(Some(application));
    window.present();
}

pub fn run_gui() -> Result<(), BoolError> {
    if let Err(e) = gtk4::init() {
        return Err(e);
    }
    let app = gtk4::Application::builder()
        .application_id("net.lobsterroast.starboard")
        .build();
    app.connect_activate(on_activate);
    let empty: Vec<String> = vec![];
    app.run_with_args(&empty);
    Ok(())
}
