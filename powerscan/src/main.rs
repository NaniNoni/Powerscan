use iced::widget::{Button, button, container, scrollable, text};
use iced::{Length, Size, Task, Theme, alignment};

use iced_aw::menu::{self, Item, Menu};
use iced_aw::{menu_bar, menu_items};

pub fn main() -> iced::Result {
    env_logger::init();

    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .window_size(Size::new(1000.0, 600.0))
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    /// Used to create a [`Task::none`].
    /// This can be useful to make disabled buttons look like clickable ones.
    None,
    /// Message to exit the app
    Quit,
}

struct App {
    title: String,
    theme: iced::Theme,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: "Powerscan".to_string(),
            theme: iced::Theme::Light,
        }
    }
}

impl App {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => Task::none(),
            Message::Quit => iced::exit(),
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let menu_from_items = |items| Menu::new(items).max_width(180.0).offset(0.0).spacing(5.0);

        #[rustfmt::skip]
        let menu_bar = menu_bar!(
            (base_button("File", Message::None), {
                menu_from_items(menu_items!(
                    (labeled_button("Quit", Message::Quit))
                ))
            })
        )
        .draw_path(menu::DrawPath::Backdrop);

        let layout = iced::widget::column![menu_bar];

        container(scrollable(layout))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

fn base_button(
    label: &str,
    msg: Message,
) -> button::Button<'_, Message, iced::Theme, iced::Renderer> {
    Button::new(text(label).align_y(alignment::Vertical::Center)).on_press(msg)
}

fn labeled_button(
    label: &str,
    msg: Message,
) -> button::Button<'_, Message, iced::Theme, iced::Renderer> {
    base_button(label, msg).width(Length::Fill)
}
