#![allow(dead_code, unused_imports)]

use {
    chrono::DateTime,
    clap::Parser,
    iced::{
        executor, font, futures,
        widget::{button, column, horizontal_space, row, text},
        Alignment, Application, Command, Element, Sandbox, Settings, Theme,
    },
    once_cell::sync::OnceCell,
    reqwest::{Method, Request},
    serde_json::Value,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Parser)]
#[clap(author, version, about)]
struct AppConfig {
    /// area code
    #[clap(short = 'a', default_value = "400")]
    area: u32,
    /// API key
    #[clap(short = 'k', long = "key", env)]
    api_key: String,
}

static CONFIG: OnceCell<AppConfig> = OnceCell::new();

// populate with area, service, api_key
const URL_TEMPLATE: &str = "https://api.nhk.or.jp/v2/pg/now";

#[derive(Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
pub enum Service {
    #[default]
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NhkView {
    service: Service,
    json: Option<Value>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    APIError,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    SwitchTo(Service),
    Reloading,
    JsonLoaded(Result<NhkView, Error>),
}

impl Application for NhkView {
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();
    type Message = Message;
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }
    fn view(&self) -> Element<Message> {
        let description_font_size = 11;
        let service = format!("{:?}", self.service);
        let on_air = self
            .json
            .as_ref()
            .and_then(|j| j.get("nowonair_list"))
            .and_then(|ch| ch.get(service));

        let following_start = on_air
            .and_then(|data| data.get("following"))
            .and_then(|data| data.get("start_time"))
            .map_or_else(|| "".to_string(), |v| v.to_string());
        let following_title = on_air
            .and_then(|data| data.get("following"))
            .and_then(|data| data.get("title"))
            .map_or_else(|| "".to_string(), |v| v.to_string());
        let following_subtitle = on_air
            .and_then(|data| data.get("following"))
            .and_then(|data| data.get("subtitle"))
            .map_or_else(|| "".to_string(), |v| v.to_string());

        let present_start = on_air
            .and_then(|data| data.get("present"))
            .and_then(|data| data.get("start_time"))
            .map_or_else(|| "".to_string(), |v| v.to_string());
        let present_title = on_air
            .and_then(|data| data.get("present"))
            .and_then(|data| data.get("title"))
            .map_or_else(|| "".to_string(), |v| v.to_string());
        let present_subtitle = on_air
            .and_then(|data| data.get("present"))
            .and_then(|data| data.get("subtitle"))
            .map_or_else(|| "".to_string(), |v| v.to_string());

        let previous_start = on_air
            .and_then(|data| data.get("previous"))
            .and_then(|data| data.get("start_time"))
            .map_or_else(|| "".to_string(), |v| v.to_string());
        let previous_title = on_air
            .and_then(|data| data.get("previous"))
            .and_then(|data| data.get("title"))
            .map_or_else(|| "".to_string(), |v| v.to_string());
        let previous_subtitle = on_air
            .and_then(|data| data.get("previous"))
            .and_then(|data| data.get("subtitle"))
            .map_or_else(|| "".to_string(), |v| v.to_string());
        column![
            row![
                button("NHK総合")
                    .width(120)
                    .padding([5, 2])
                    .on_press(Message::SwitchTo(Service::G1)),
                button("Eテレ")
                    .width(120)
                    .padding([5, 2])
                    .on_press(Message::SwitchTo(Service::E1)),
                button("NHK FM")
                    .width(120)
                    .padding([5, 2])
                    .on_press(Message::SwitchTo(Service::R3)),
                button("ラジオ第1")
                    .width(120)
                    .padding([5, 2])
                    .on_press(Message::SwitchTo(Service::R1)),
                button("ラジオ第2")
                    .width(120)
                    .padding([5, 2])
                    .on_press(Message::SwitchTo(Service::R2)),
            ]
            .spacing(5)
            // .padding(5)
            .align_items(Alignment::Center),
            row![
                text("開始時刻").width(100),
                horizontal_space(30),
                text("タイトル").width(200),
                horizontal_space(30),
                text("内容").width(300),
                horizontal_space(30),
            ],
            row![
                text(following_start).size(description_font_size).width(100),
                horizontal_space(30),
                text(following_title).size(description_font_size).width(200),
                horizontal_space(30),
                text(following_subtitle)
                    .size(description_font_size)
                    .width(300),
                horizontal_space(30),
            ],
            row![
                text(present_start).size(description_font_size).width(100),
                horizontal_space(30),
                text(present_title).size(description_font_size).width(200),
                horizontal_space(30),
                text(present_subtitle)
                    .size(description_font_size)
                    .width(300),
                horizontal_space(30),
            ],
            row![
                text(previous_start).size(description_font_size).width(100),
                horizontal_space(30),
                text(previous_title).size(description_font_size).width(200),
                horizontal_space(30),
                text(previous_subtitle)
                    .size(description_font_size)
                    .width(300),
                horizontal_space(30),
            ],
        ]
        .spacing(10)
        .into()
    }
    fn title(&self) -> String {
        String::from("NHK now")
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SwitchTo(service) => {
                Command::perform(NhkView::get_data(service), Message::JsonLoaded)
            }
            Message::Reloading => Command::none(),
            Message::JsonLoaded(Ok(data)) => {
                *self = data;
                Command::none()
            }
            _ => Command::none(),
        }
    }
}

impl NhkView {
    async fn get_data(service: Service) -> Result<Self, Error> {
        let Some(config) = CONFIG.get() else {
            panic!();
        };
        let url = format!(
            "{}/{}/{:?}.json?key={}",
            URL_TEMPLATE, config.area, service, config.api_key
        );
        let Ok(text) = &reqwest::get(url).await.ok().unwrap().text().await else {
            panic!();
        };
        Ok(Self {
            service,
            json: dbg!(serde_json::from_str(text).ok().unwrap()),
        })
    }
}

fn main() -> iced::Result {
    let config = AppConfig::parse();
    if CONFIG.get().is_none() {
        CONFIG.set(config.clone()).expect("fail to store config");
    }
    let mut settings = Settings::default();
    settings.default_font.family = font::Family::Name("ヒラギノ角ゴシック");
    settings.default_text_size = 16.0;
    settings.window.size = (620, 240);
    NhkView::run(settings)
}
