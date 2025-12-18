use gtk4::{Application, Builder, IconTheme, gdk, prelude::*};

use gio::{
    ffi::{GResource, g_resources_register},
    glib::{BoolError, ExitCode},
};

#[link(name = "resources")]
unsafe extern "C" {
    fn gresource_get_resource() -> *mut GResource;
}

pub struct GtkWrapper {
    app: Application,
    builder: Builder,
}

impl GtkWrapper {
    pub fn new() -> Result<Self, BoolError> {
        unsafe {
            let resources = gresource_get_resource();
            g_resources_register(resources)
        }

        if let Err(e) = gtk4::init() {
            return Err(e);
        }

        let app = gtk4::Application::builder()
            .application_id("net.lobsterroast.starboard")
            .build();

        activate_css();
        let src = include_str!("gtk/layout.ui");
        let builder = Builder::from_string(src);

        let display = gdk::Display::default().expect("Unable to get default display.");
        let icon_theme = IconTheme::for_display(&display);
        icon_theme.add_resource_path("/github/lobsterroast/starboard");

        app.connect_activate(gio::glib::clone!(
            #[strong]
            builder,
            move |app| on_activate(app, &builder)
        ));

        Ok(Self {
            app: app,
            builder: builder,
        })
    }

    pub fn run(&self) -> bool {
        let empty: Vec<String> = vec![];
        return self.app.run_with_args(&empty) == ExitCode::SUCCESS;
    }
}

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

fn on_activate(application: &gtk4::Application, builder: &Builder) {
    let window = builder
        .object::<gtk4::ApplicationWindow>("window")
        .expect("Unable to parse GTK root element.");

    window.set_application(Some(application));
    window.present();
}
