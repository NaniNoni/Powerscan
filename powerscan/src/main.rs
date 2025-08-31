use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4 as gtk;
use sane::Sane;

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

        let sane = Sane::init(0).unwrap();
        let device_list = sane.get_devices().unwrap();
        for device in device_list {
            println!("{:?}", device);
        }

        // Show the window.
        window.present();
    });

    app.run()
}
