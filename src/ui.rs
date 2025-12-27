use gtk4::{Application, Builder, Button, IconTheme, Label, gdk, prelude::*};

use gio::{
    SimpleAction, SimpleActionGroup,
    ffi::{GResource, g_resources_register},
    glib::{self, BoolError, ExitCode},
};

use tokio::{io::AsyncReadExt, net::UnixStream};

use bincode::{config::Configuration, decode_from_slice};

use crate::ui::glib::GString;
use crate::util::InputPacket;

#[link(name = "resources")]
unsafe extern "C" {
    fn gresource_get_resource() -> *mut GResource;
}

async fn read_sock_loop(builder: Builder) {
    // Because the starboard server and the manager are run as different
    // processes, they need to have some sort of IPC. We will use a Unix
    // Domain Socket for this the socket 'starboard.sock' will be the
    // bridge between the starboard server daemon and the GUI manager
    // displaying debug information.

    let mut sock = UnixStream::connect("/tmp/starboard.sock")
        .await
        .expect("Unable to connect to /tmp/starboard.sock");

    // because get_input_packet is a future that is being awaited, it will yield
    // control back to the glib executor instead of blocking.

    loop {
        let packet = get_input_packet(&mut sock).await;
        let builder_clone = builder.clone();
        update_info(builder_clone, packet);
    }
}

async fn get_input_packet(sock: &mut UnixStream) -> InputPacket {
    // UnixSteam.read() requires a struct that implements the trait 'MutBuf'.
    // This is not implemented by &[u8] so Vec<u8> must be used instead.

    let mut buf: Vec<u8> = Vec::new();
    buf.resize(512, 0);

    // Control is yielded back to glib executor here
    let _ = sock.read(&mut buf).await;

    let conf = Configuration::default();
    decode_from_slice::<InputPacket, Configuration>(&buf, conf)
        .unwrap()
        .0
}

fn update_info(builder: Builder, packet: InputPacket) {
    for (i, val) in packet.abs_states.iter().enumerate() {
        let analog_name = format!("abs_{}", i);
        let analog_str: GString = format!("Current State: {}", val).into();
        let builder_clone = builder.clone();
        update_widget_state(builder_clone, analog_name, analog_str);
    }

    for (i, val) in packet.key_states_as_arr().iter().enumerate() {
        let bin_name = format!("bin_{}", i);
        let bin_str: GString = format!("Pressed: {}", val).into();
        let builder_clone = builder.clone();
        update_widget_state(builder_clone, bin_name, bin_str);
    }
}

fn update_widget_state(builder: Builder, widget_name: String, widget_str: GString) {
    if let Some(button) = builder.object::<Button>(&widget_name) {
        if let Some(mut label) = button.label() {
            label = widget_str;
        }
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

    let action_group = SimpleActionGroup::new();
    let activate_starboard_action = SimpleAction::new("activate-starboard", None);
    activate_starboard_action.connect_activate(glib::clone!(
        #[strong]
        builder,
        move |_, _| activate_starboard_callback(&builder)
    ));
    action_group.add_action(&activate_starboard_action);
    activate_starboard_action.set_enabled(true);

    window.insert_action_group("actions", Some(&action_group));
    window.set_application(Some(application));
    window.present();
}

fn activate_starboard_callback(builder: &Builder) {
    if let Some(label) = builder.object::<Label>("status-label") {
        label.set_text("Skibidi");
    }
}

pub struct GtkWrapper {
    app: Application,
    builder: Builder,
}

impl GtkWrapper {
    pub async fn new() -> Result<Self, BoolError> {
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
        glib::spawn_future_local(read_sock_loop(self.builder.clone()));
        return self.app.run_with_args(&empty) == ExitCode::SUCCESS;
    }
}
