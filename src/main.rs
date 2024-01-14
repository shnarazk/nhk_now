#![allow(dead_code, unused_imports)]

use {
    chrono::DateTime,
    clap::Parser,
    iced::{
        executor, font,
        widget::{button, column, horizontal_space, row, text},
        Alignment, Application, Command, Element, Sandbox, Settings, Theme,
    },
    // nhk_now::reqwest_plugin::*,
    serde_json::Value,
};

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Service {
    None,
    G1,
    E1,
    R1,
    R2,
    R3,
}

impl std::fmt::Debug for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Service::None => "",
                Service::G1 => "g1",
                Service::E1 => "e1",
                Service::R1 => "r1",
                Service::R2 => "r2",
                Service::R3 => "r3",
            }
        )
    }
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Service::None => "",
                Service::G1 => "NHK総合1",
                Service::E1 => "NHKEテレ1",
                Service::R1 => "NHKラジオ第1",
                Service::R2 => "NHKラジオ第2",
                Service::R3 => "NHK FM",
            }
        )
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum Timeline {
    Following,
    Present,
    Previous,
}

impl std::fmt::Debug for Timeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Timeline::Following => "following",
                Timeline::Present => "present",
                Timeline::Previous => "previous",
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
enum Description {
    StartTime,
    Title,
    Subtitle,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Parser)]
#[clap(author, version, about)]
struct AppConfig {
    /// area code
    #[clap(short = 'a', default_value = "400")]
    area: u32,
    /// API key
    #[clap(short = 'k', long = "key", env)]
    nhk_api_key: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Counter {
    value: isize,
    json: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Message {
    SwitchTo(Service),
    Reloading,
    JsonLoaded,
}

impl Application for Counter {
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();
    type Message = Message;
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }
    fn view(&self) -> Element<Message> {
        column![
            row![
                button("NHK総合")
                    .padding([10, 5])
                    .on_press(Message::SwitchTo(Service::G1)),
                button("NHKEテレ")
                    .padding([10, 5])
                    .on_press(Message::SwitchTo(Service::E1)),
                button("NHK FM")
                    .padding([10, 5])
                    .on_press(Message::SwitchTo(Service::R3)),
                button("NHK Radio第1")
                    .padding([10, 5])
                    .on_press(Message::SwitchTo(Service::R1)),
                button("NHK Radio第2")
                    .padding([10, 5])
                    .on_press(Message::SwitchTo(Service::R2)),
            ]
            .spacing(5)
            .padding(5)
            .align_items(Alignment::Center),
            row![
                text(""),
                horizontal_space(30),
                text("タイトル"),
                horizontal_space(30),
                text("内容"),
                horizontal_space(30),
            ],
            row![
                text("次番組"),
                horizontal_space(30),
                text("News7"),
                horizontal_space(30),
                text("いろいろなニュースと気象予報"),
                horizontal_space(30),
            ],
            row![
                text("現番組"),
                horizontal_space(30),
                text(""),
                horizontal_space(30),
                text(""),
                horizontal_space(30),
            ],
            row![
                text("前番組"),
                horizontal_space(30),
                text(""),
                horizontal_space(30),
                text(""),
                horizontal_space(30),
            ],
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
    fn title(&self) -> String {
        String::from("NHK now")
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            _ => (),
        }
        Command::none()
    }
}

fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.default_font.family = font::Family::Name("ヒラギノ角ゴシック");
    settings.default_text_size = 18.0;
    settings.window.size = (640, 280);
    Counter::run(settings)
}
