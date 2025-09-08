use log::{debug, error};
use relm4::gtk::prelude::*;
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::*;
use relm4::{AsyncComponentSender, RelmApp, RelmWidgetExt, gtk, view};
use sane::{SANE_Status, Sane, SaneError};

struct AppModel {
    sane: Sane,
    devices: Vec<sane::Device>,
}

#[derive(Debug)]
enum AppMsg {
    StartScan,
    ScanError(SaneError),
    ScanFinished(Vec<u8>),
}

#[relm4::component(async)]
impl AsyncComponent for AppModel {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_title: Some("Powerscan"),
            set_default_width: 300,
            set_default_height: 100,

            gtk::Box {
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button::with_label("Scan") {
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::StartScan);
                    }
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("Devices: {:?}", model.devices),
                    set_margin_all: 5,
                }
            }
        }
    }

    fn init_loading_widgets(root: Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local]
            root {
                set_title: Some("Simple app"),
                set_default_size: (300, 100),

                // This will be removed automatically by
                // LoadingWidgets when the full view has loaded
                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    // Initialize the component.
    async fn init(
        _counter: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let (sane, devices) = relm4::spawn(async move {
            // TODO: error handling
            let sane = Sane::init().unwrap();
            let devices: Vec<sane::Device> = sane.get_devices().unwrap();

            (sane, devices)
        })
        .await
        .unwrap();

        let model = AppModel { sane, devices };

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppMsg::StartScan => {
                // let devices = self.devices.clone();
                let sane = self.sane.clone();

                relm4::spawn(async move {
                    const CHUNK_SIZE: usize = 512;
                    let handle = sane.open("genesys:libusb:001:007").unwrap();
                    handle.start().unwrap();

                    let mut data = Vec::new();
                    loop {
                        match handle.read(CHUNK_SIZE) {
                            Ok(chunk) => {
                                println!("Chunk: {chunk:?}");
                                data.extend_from_slice(&chunk);
                            }
                            Err(SaneError::InternalSANE { status }) => {
                                if status == SANE_Status::SANE_STATUS_EOF {
                                    break;
                                }
                            }
                            Err(e) => sender.input(AppMsg::ScanError(e)),
                        }
                    }

                    sender.input(AppMsg::ScanFinished(data))
                })
                .await
                .unwrap()
            }
            AppMsg::ScanError(e) => {
                error!("Error while scanning: {e}");
            }
            AppMsg::ScanFinished(data) => {
                debug!("Finished scanning: {data:?}");
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.test.simple_manual");
    app.run_async::<AppModel>(0);
}
