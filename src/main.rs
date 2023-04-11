use {
    bevy::{prelude::*, winit::WinitSettings},
    chrono::DateTime,
    clap::Parser,
    nhk_now::reqwest_plugin::*,
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

#[derive(Debug, Resource)]
struct CurrentService(Service);

#[derive(Debug, Component)]
struct TargetService(Service);

#[derive(Debug, Resource)]
struct ReqestTicket(bool);

#[derive(Component, Clone, Eq, PartialEq, PartialOrd, Ord)]
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
impl Timeline {
    fn style(&self) -> BackgroundColor {
        match self {
            Timeline::Following => BackgroundColor(Color::Hsla {
                hue: 0.1,
                saturation: 0.2,
                lightness: 1.0,
                alpha: 1.0,
            }),
            // Timeline::Following => "bg-slate-100 text-gray-600",
            Timeline::Present => BackgroundColor(Color::Hsla {
                hue: 0.3,
                saturation: 0.3,
                lightness: 0.9,
                alpha: 1.0,
            }),
            // Timeline::Present => "bg-slate-200 text-black",
            Timeline::Previous => BackgroundColor(Color::Hsla {
                hue: 0.4,
                saturation: 0.7,
                lightness: 0.6,
                alpha: 1.0,
            }),
            // Timeline::Previous => "bg-slate-400 text-gray-800",
        }
    }
}

#[derive(Component, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
enum Description {
    StartTime,
    Title,
    Subtitle,
}
const ACTIVE_CHANNEL_COLOR: Color = Color::rgb(1., 0.866, 0.849);
const JUSTIFY_CONTENT_COLOR: Color = Color::rgb(0.802, 0.922, 1.);
// const JUSTIFY_CONTENT_COLOR: Color = Color::rgb(0.102, 0.522, 1.);
const MARGIN: Val = Val::Px(2.);

#[derive(Clone, Debug, Default, Eq, PartialEq, Parser, Resource)]
#[clap(author, version, about)]
struct AppConfig {
    /// area code
    #[clap(short = 'a', default_value = "400")]
    area: u32,
    /// API key
    #[clap(short = 'k', long = "key", env)]
    apikey: String,
}

fn main() {
    let app_config = AppConfig::parse();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: [700., 350.].into(),
                title: "NHK now".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugin(ReqwestPlugin)
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(app_config)
        .insert_resource(CurrentService(Service::None))
        .insert_resource(ReqestTicket(false))
        .add_systems(Startup, spawn_layout)
        .add_systems(Update, (button_system, send_requests, handle_responses))
        .run()
}

fn spawn_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/NotoSansCJKjp-Regular.otf");
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                // fill the entire window
                size: Size::all(Val::Percent(100.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..Default::default()
        })
        .with_children(|builder| {
            // spawn the key
            builder
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::top(MARGIN),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|builder| {
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        UiRect::right(MARGIN),
                        Service::G1,
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        UiRect::right(MARGIN),
                        Service::E1,
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        UiRect::right(MARGIN),
                        Service::R1,
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        UiRect::right(MARGIN),
                        Service::R2,
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        UiRect::right(MARGIN),
                        Service::R3,
                    );
                });

            builder
                .spawn(NodeBundle {
                    style: Style {
                        min_size: Size::new(Val::Percent(96.), Val::Percent(100.)),
                        flex_direction: FlexDirection::Column,
                        // flex_direction: FlexDirection::Row,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|builder| {
                    spawn_timeline_text_bundle(builder, font.clone(), Timeline::Following);
                    spawn_timeline_text_bundle(builder, font.clone(), Timeline::Present);
                    spawn_timeline_text_bundle(builder, font.clone(), Timeline::Previous);
                });
        });
}

fn spawn_timeline_text_bundle(builder: &mut ChildBuilder, font: Handle<Font>, timeline: Timeline) {
    builder
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                size: Size::new(Val::Percent(100.), Val::Percent(14.)),
                // margin,
                padding: UiRect {
                    top: Val::Px(3.),
                    left: Val::Px(5.),
                    right: Val::Px(5.),
                    bottom: Val::Px(2.),
                },
                ..Default::default()
            },
            background_color: timeline.style(), // BackgroundColor(background_color),
            ..Default::default()
        })
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::FlexStart,
                        size: Size::new(Val::Percent(100.), Val::Percent(10.)),
                        //  margin,
                        padding: UiRect {
                            top: Val::Px(1.),
                            left: Val::Px(5.),
                            right: Val::Px(5.),
                            bottom: Val::Px(1.),
                        },
                        ..Default::default()
                    },
                    background_color: timeline.style(), // BackgroundColor(background_color),
                    ..Default::default()
                })
                .with_children(|builder| {
                    builder.spawn((
                        TextBundle::from_section(
                            "開始時刻",
                            TextStyle {
                                font: font.clone(),
                                font_size: 20.0,
                                color: Color::BLACK,
                            },
                        )
                        .with_style(Style {
                            size: Size::new(Val::Percent(15.0), Val::Px(80.0)),
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        }),
                        timeline.clone(),
                        Description::StartTime,
                    ));
                    builder.spawn((
                        TextBundle::from_section(
                            "タイトル",
                            TextStyle {
                                font: font.clone(),
                                font_size: 20.0,
                                color: Color::BLACK,
                            },
                        )
                        .with_style(Style {
                            size: Size::new(Val::Percent(85.0), Val::Px(80.0)),
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        }),
                        timeline.clone(),
                        Description::Title,
                    ));
                });
        });
    builder
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::Wrap,
                // justify_content: JustifyContent::Center,
                justify_content: JustifyContent::FlexStart,
                size: Size::new(Val::Percent(100.), Val::Percent(15.)),
                // margin,
                padding: UiRect {
                    top: Val::Px(1.),
                    left: Val::Px(35.),
                    right: Val::Px(5.),
                    bottom: Val::Px(1.),
                },
                ..Default::default()
            },
            background_color: timeline.style(),
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section(
                    "番組内容\n表示パネル",
                    TextStyle {
                        font: font.clone(),
                        font_size: 18.0,
                        color: Color::BLACK,
                    },
                ),
                timeline.clone(),
                Description::Subtitle,
            ));
        });
}

fn spawn_styled_button_bundle(
    builder: &mut ChildBuilder,
    font: Handle<Font>,
    margin: UiRect,
    service: Service,
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                margin,
                flex_direction: FlexDirection::Row,
                padding: UiRect {
                    top: Val::Px(1.),
                    left: Val::Px(5.),
                    right: Val::Px(5.),
                    bottom: Val::Px(1.),
                },
                ..Default::default()
            },
            // background_color: BackgroundColor(background_color),
            ..Default::default()
        })
        .with_children(|builder| {
            builder
                .spawn((
                    ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(120.0), Val::Px(30.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(JUSTIFY_CONTENT_COLOR),
                        ..default()
                    },
                    TargetService(service.clone()),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!("{}", service),
                        TextStyle {
                            font,
                            font_size: 20.0,
                            color: Color::BLACK,
                        },
                    ));
                });
        });
}

// const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
// const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
type ButtonLike = (Changed<Interaction>, With<Button>);

fn button_system(
    mut current_service: ResMut<CurrentService>,
    mut triggered: ResMut<ReqestTicket>,
    interaction_query: Query<(&Interaction, &TargetService), ButtonLike>,
) {
    for (interaction, target) in &interaction_query {
        if *interaction == Interaction::Clicked {
            current_service.0 = target.0.clone();
            triggered.0 = true;
        }
    }
}

fn parse_json(json: &Value) -> Option<(Value, String)> {
    if let Some(list) = json.get("nowonair_list") {
        for ch in ["g1", "e1", "r1", "r2", "r3"] {
            if let Some(target) = list.get(ch) {
                return Some((target.clone(), ch.to_string()));
            }
        }
    }
    if json.get("NO").is_some() {
        return Some((json.clone(), "".to_string()));
    }
    None
}

fn send_requests(
    config: Res<AppConfig>,
    current_service: Res<CurrentService>,
    mut triggered: ResMut<ReqestTicket>,
    mut commands: Commands,
    mut interaction_query: Query<&mut BackgroundColor, With<Button>>,
) {
    if !triggered.0 {
        return;
    }
    triggered.0 = false;
    if current_service.0 == Service::None {
        return;
    }
    let Ok(base) = format!(
        "https://api.nhk.or.jp/v2/pg/now/{}/{:?}.json?key={}",
        config.area, current_service.0, config.apikey,
    ).as_str().try_into() else {
        return;
    };
    let req = reqwest::Request::new(reqwest::Method::GET, base);
    commands.spawn(ReqwestRequest(Some(req)));
    for mut color in &mut interaction_query {
        *color = JUSTIFY_CONTENT_COLOR.into();
    }
}

fn handle_responses(
    mut commands: Commands,
    results: Query<(Entity, &ReqwestBytesResult)>,
    current_service: Res<CurrentService>,
    mut buttons: Query<(&TargetService, &mut BackgroundColor), With<Button>>,
    mut timelines: Query<(&Timeline, &Description, &mut Text)>,
) {
    for (e, res) in results.iter() {
        let string = res.as_str().unwrap();
        let json: Value = serde_json::from_str(string).expect("invalid data received");
        let Some((data, _)) = parse_json(&json) else {
            return;
        };

        // update button colors
        for (service, mut color) in &mut buttons {
            if current_service.0 == service.0 {
                *color = ACTIVE_CHANNEL_COLOR.into();
            }
        }
        // update button colors and contents table
        for (timeline, description, mut text) in &mut timelines {
            // match *timeline {
            //     Timeline::Following => {
            //         text.sections[0].value = data[format!("{timeline:?}")]["title"].to_string();
            //     }
            //     Timeline::Present => {
            //         dbg!(&description);
            //         text.sections[0].value = data[format!("{timeline:?}")]["title"].to_string();
            //     }
            //     Timeline::Previous => {
            //         text.sections[0].value = data[format!("{timeline:?}")]["title"].to_string();
            //     }
            // }
            match description {
                Description::StartTime => {
                    text.sections[0].value = DateTime::parse_from_rfc3339(
                        data[format!("{timeline:?}")]["start_time"]
                            .as_str()
                            .unwrap(),
                    )
                    .unwrap()
                    .format("%H:%M")
                    .to_string();
                }
                Description::Title => {
                    text.sections[0].value =
                        unquote(false, &data[format!("{timeline:?}")]["title"]);
                }
                Description::Subtitle => {
                    text.sections[0].value =
                        unquote(true, &data[format!("{timeline:?}")]["subtitle"]);
                }
            }
        }
        // Done with this entity
        commands.entity(e).despawn_recursive();
    }
}

fn unquote(line_break: bool, s: &Value) -> String {
    if let Some(s) = s.as_str() {
        let mut t = String::new();
        let at = 50;
        for (i, c) in s.chars().enumerate() {
            t.push(c);
            if line_break && i % at == at - 1 {
                t.push('\n');
            }
        }
        t
    } else {
        String::new()
    }
}
