#![allow(dead_code, unused_imports)]
use {
    chrono::DateTime,
    clap::Parser,
    iced::{
        font,
        widget::{button, column, horizontal_space, row, text},
        Alignment, Element, Sandbox, Settings,
    },
    // nhk_now::reqwest_plugin::*,
    serde_json::Value,
};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum Service {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for Counter {
    type Message = Message;
    fn new() -> Self {
        Self::default()
    }
    fn view(&self) -> Element<Message> {
        column![
            row![
                button("NHK総合")
                    .padding([20, 25])
                    .on_press(Message::DecrementPressed),
                button("NHKEテレ")
                    .padding([20, 25])
                    .on_press(Message::DecrementPressed),
                button("NHK FM")
                    .padding([20, 25])
                    .on_press(Message::DecrementPressed),
                button("NHK Radio第1")
                    .padding([20, 25])
                    .on_press(Message::DecrementPressed),
                button("情報更新")
                    .padding([20, 25])
                    .on_press(Message::DecrementPressed),
            ]
            .spacing(10)
            .padding(20)
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
                text(""),
                horizontal_space(30),
                text(""),
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
        String::from("Counter -- iced")
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => self.value += 1,
            Message::DecrementPressed => self.value -= 1,
        }
    }
}

fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.default_font.family = font::Family::Name("ヒラギノ角ゴシック");
    settings.default_text_size = 18.0;
    Counter::run(settings)
}
