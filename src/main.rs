use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4 as gtk;

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
    });

    app.run()
}
