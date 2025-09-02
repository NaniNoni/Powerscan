use std::sync::OnceLock;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4 as gtk;
use gtk4::glib::clone;
use sane::{Device, Sane, SaneError};
use tokio::runtime::Runtime;

fn tokio_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("Setting up tokio runtime needs to succeed."))
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("com.github.naninoni.powerscan")
        .build();

    app.connect_activate(|app| {
        // We create the main window.
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(320)
            .default_height(200)
            .title("Hello, World!")
            .build();

        // Show the window.
        window.present();

        let (sender, receiver) = async_channel::bounded::<Result<Vec<Device>, SaneError>>(1);
        tokio_runtime().spawn(clone!(
            #[strong]
            sender,
            async move {
                let device_list = Sane::init(0).map_or_else(Err, |sane| sane.get_devices());
                sender
                    .send(device_list)
                    .await
                    .expect("Failed to open channel");
            }
        ));

        glib::spawn_future_local(async move {
            while let Ok(device_list) = receiver.recv().await {
                match device_list {
                    Ok(device_list) => {
                        for device in device_list {
                            println!("{:?}", device);
                        }
                    }
                    Err(e) => {
                        eprintln! {"{e}"};
                    }
                }
            }
        });
    });

    app.run()
}
