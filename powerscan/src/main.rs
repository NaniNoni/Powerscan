use std::sync::OnceLock;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4::glib::clone;
use gtk4::{self as gtk, CenterBox, Spinner};
use log::{error, info};
use sane::{Device, Sane, SaneError};
use tokio::runtime::Runtime;

fn tokio_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("Setting up tokio runtime needs to succeed."))
}

fn main() -> glib::ExitCode {
    env_logger::init();

    let app = Application::builder()
        .application_id("com.github.naninoni.powerscan")
        .build();

    app.connect_activate(|app| {
        let center_box = CenterBox::new();
        let spinner = Spinner::new();
        center_box.set_center_widget(Some(&spinner));

        // We create the main window.
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(320)
            .default_height(200)
            .title("Powerscan")
            .child(&center_box)
            .build();

        // Show the window.
        window.present();

        spinner.start();
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
                            info!("{:?}", device);
                        }
                    }
                    Err(e) => {
                        error!("Error retreaving SANE device list: {:?}", e)
                    }
                }

                spinner.stop();
            }
        });
    });

    app.run()
}
