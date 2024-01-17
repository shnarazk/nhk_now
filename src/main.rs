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
                Service::E1 => "Eテレ1",
                Service::R1 => "ラジオ第1",
                Service::R2 => "ラジオ第2",
                Service::R3 => "NHK FM",
            }
        )
    }
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

trait ParseOnAirJson {
    fn get_content(&self, timeline: &str) -> (String, String, String);
}

impl ParseOnAirJson for Option<&Value> {
    fn get_content(&self, timeline: &str) -> (String, String, String) {
        let description = self.and_then(|data| data.get(timeline));
        let start = description
            .and_then(|data| data.get("start_time"))
            .map_or_else(
                || "".to_string(),
                |v| {
                    v.to_string()
                        .trim_matches('"')
                        .chars()
                        .skip(11)
                        .take(5)
                        .collect::<String>()
                },
            );
        let title = description.and_then(|data| data.get("title")).map_or_else(
            || "".to_string(),
            |v| v.to_string().trim_matches('"').to_string(),
        );
        let subtitle = description
            .and_then(|data| data.get("subtitle"))
            .map_or_else(
                || "".to_string(),
                |v| v.to_string().trim_matches('"').to_string(),
            );
        (start, title, subtitle)
    }
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
        let on_air = self
            .json
            .as_ref()
            .and_then(|j| j.get("nowonair_list"))
            .and_then(|ch| ch.get(format!("{:?}", self.service)));
        let following = on_air.get_content("following");
        let present = on_air.get_content("present");
        let previous = on_air.get_content("previous");
        macro_rules! row1 {
            ($value: expr) => {
                row![
                    text($value.0).width(50),
                    horizontal_space(30),
                    text($value.1).width(540),
                ]
                .padding(4)
            };
        }
        macro_rules! row2 {
            ($value: expr) => {
                row![
                    horizontal_space(30),
                    text($value.2).size(description_font_size).width(550),
                    horizontal_space(30),
                ]
                .align_items(Alignment::Start)
            };
        }
        macro_rules! button_color {
            ($service: expr) => {
                if self.service == $service {
                    iced::theme::Button::Positive
                } else {
                    iced::theme::Button::Secondary
                }
            };
        }
        macro_rules! button {
            ($name: expr, $service: expr) => {
                button($name)
                    .width(120)
                    .padding([5, 2])
                    .style(button_color!($service))
                    .on_press(Message::SwitchTo($service))
                    .into()
            };
        }
        column![
            row(vec![
                button!("NHK 総合", Service::G1),
                button!("NHK Eテレ", Service::E1),
                button!("NHK FM", Service::R3),
                button!("ラジオ第1", Service::R1),
                button!("ラジオ第2", Service::R2),
            ])
            .spacing(4)
            // .padding(5)
            .align_items(Alignment::Center),
            row1!(following),
            row2!(following),
            row1!(present),
            row2!(present),
            row1!(previous),
            row2!(previous),
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
            json: serde_json::from_str(text).ok().unwrap(),
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
    settings.default_text_size = 13.0;
    settings.window.size = (620, 270);
    NhkView::run(settings)
}
