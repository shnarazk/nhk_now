use {
    bevy::{
        prelude::*,
        // tasks::{AsyncComputeTaskPool, Task},
        winit::WinitSettings,
    },
    // chrono::DateTime,
    clap::Parser,
    nhk_now::reqwest_plugin::*,
    // hyper::Client,
    // hyper_tls::HttpsConnector,
    serde_json::Value,
};

#[derive(Component, Clone, Eq, PartialEq, PartialOrd, Ord)]
enum CurrentService {
    None,
    G1,
    E1,
    R1,
    R2,
    R3,
}
impl std::fmt::Debug for CurrentService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CurrentService::None => "",
                CurrentService::G1 => "g1",
                CurrentService::E1 => "e1",
                CurrentService::R1 => "r1",
                CurrentService::R2 => "r2",
                CurrentService::R3 => "r3",
            }
        )
    }
}

impl std::fmt::Display for CurrentService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CurrentService::None => "",
                CurrentService::G1 => "NHK総合1",
                CurrentService::E1 => "NHKEテレ1",
                CurrentService::R1 => "NHKラジオ第1",
                CurrentService::R2 => "NHKラジオ第2",
                CurrentService::R3 => "NHK FM",
            }
        )
    }
}

#[allow(dead_code)]
const TIMELINE: [(&str, &str); 3] = [
    ("following", "bg-slate-100 text-gray-600"),
    ("present", "bg-slate-200 text-black"),
    ("previous", "bg-slate-400 text-gray-800"),
];
const ACTIVE_CHANNEL_COLOR: Color = Color::rgb(1., 0.066, 0.349);
const JUSTIFY_CONTENT_COLOR: Color = Color::rgb(0.102, 0.522, 1.);
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
                resolution: [700., 440.].into(),
                title: "NHK now".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugin(ReqwestPlugin)
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(app_config)
        .insert_resource(ReqestTicket(CurrentService::None))
        .add_systems(Startup, spawn_layout)
        .add_systems(
            Update,
            (
                button_system,
                send_requests,
                handle_responses,
                button_system2,
            ),
        )
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
                        ACTIVE_CHANNEL_COLOR,
                        UiRect::right(MARGIN),
                        CurrentService::G1,
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        CurrentService::E1,
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        CurrentService::R1,
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        CurrentService::R2,
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        CurrentService::R3,
                    );
                });

            builder
                .spawn(NodeBundle {
                    style: Style {
                        min_size: Size::new(Val::Px(850.), Val::Px(1020.)),
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|_builder| {
                    // spawn one child node for each combination of `AlignItems` and `JustifyContent`
                    // let justifications = [
                    //     JustifyContent::FlexStart,
                    //     JustifyContent::Center,
                    //     JustifyContent::FlexEnd,
                    //     JustifyContent::SpaceEvenly,
                    //     JustifyContent::SpaceAround,
                    //     JustifyContent::SpaceBetween,
                    // ];
                    // let alignments = [
                    //     AlignItems::Baseline,
                    //     AlignItems::FlexStart,
                    //     AlignItems::Center,
                    //     AlignItems::FlexEnd,
                    //     AlignItems::Stretch,
                    // ];
                    // builder
                    //     .spawn(NodeBundle {
                    //         style: Style {
                    //             flex_direction: FlexDirection::Row,
                    //             ..Default::default()
                    //         },
                    //         ..Default::default()
                    //     })
                    //     .with_children(|builder| {
                    //         spawn_child_node(
                    //             builder,
                    //             font.clone(),
                    //             AlignItems::Baseline,
                    //             JustifyContent::Center,
                    //         );
                    //     });
                });
        });
}

#[allow(dead_code)]
fn spawn_child_node(
    builder: &mut ChildBuilder,
    font: Handle<Font>,
    align_items: AlignItems,
    justify_content: JustifyContent,
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items,
                justify_content,
                size: Size::all(Val::Px(160.)),
                margin: UiRect::all(MARGIN),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::DARK_GRAY),
            ..Default::default()
        })
        .with_children(|builder| {
            let labels = [
                ("align_items:NHKナウ", ACTIVE_CHANNEL_COLOR, 0.),
                ("justify_content:チャンネル", JUSTIFY_CONTENT_COLOR, 3.),
                // (format!("{align_items:?}"), ACTIVE_CHANNEL_COLOR, 0.),
                // (format!("{justify_content:?}"), JUSTIFY_CONTENT_COLOR, 3.),
            ];
            for (text, color, top_margin) in labels {
                // We nest the text within a parent node because margins and padding can't be directly applied to text nodes currently.
                spawn_nested_text_bundle(
                    builder,
                    font.clone(),
                    color,
                    UiRect::top(Val::Px(top_margin)),
                    text,
                );
            }
        });
}

fn spawn_nested_text_bundle(
    builder: &mut ChildBuilder,
    font: Handle<Font>,
    background_color: Color,
    margin: UiRect,
    text: &str,
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                margin,
                padding: UiRect {
                    top: Val::Px(1.),
                    left: Val::Px(5.),
                    right: Val::Px(5.),
                    bottom: Val::Px(1.),
                },
                ..Default::default()
            },
            background_color: BackgroundColor(background_color),
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font,
                    font_size: 24.0,
                    color: Color::BLACK,
                },
            ));
        });
}

fn spawn_styled_button_bundle(
    builder: &mut ChildBuilder,
    font: Handle<Font>,
    background_color: Color,
    margin: UiRect,
    service: CurrentService,
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
                        background_color: BackgroundColor(background_color),
                        ..default()
                    },
                    service.clone(),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!("{}", service),
                        TextStyle {
                            font,
                            font_size: 24.0,
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
    mut current_service: ResMut<ReqestTicket>,
    interaction_query: Query<(&Interaction, &CurrentService), ButtonLike>,
) {
    for (interaction, target) in &interaction_query {
        if *interaction == Interaction::Clicked {
            current_service.0 = target.clone();
        }
    }
}

fn button_system2(
    current_service: Res<ReqestTicket>,
    mut interaction_query: Query<(&CurrentService, &mut BackgroundColor), With<Button>>,
) {
    for (service, mut color) in &mut interaction_query {
        *color = if &current_service.0 == service {
            ACTIVE_CHANNEL_COLOR.into()
        } else {
            JUSTIFY_CONTENT_COLOR.into()
        };
    }
}

#[allow(dead_code)]
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

#[derive(Debug, Resource)]
struct ReqestTicket(CurrentService);

fn send_requests(config: Res<AppConfig>, mut commands: Commands, mut ch: ResMut<ReqestTicket>) {
    if ch.0 == CurrentService::None {
        return;
    }
    let Ok(base) = dbg!(format!(
        "https://api.nhk.or.jp/v2/pg/now/{}/{:?}.json?key={}",
        400, ch.0, config.apikey,
    )).as_str().try_into() else {
        return;
    };
    let req = reqwest::Request::new(reqwest::Method::GET, base);
    ch.0 = CurrentService::None;
    commands.spawn(ReqwestRequest(Some(req)));
    dbg!();
}

fn handle_responses(mut commands: Commands, results: Query<(Entity, &ReqwestBytesResult)>) {
    for (e, res) in results.iter() {
        let string = res.as_str().unwrap();
        let json: Value = serde_json::from_str(string).expect("invalid json");
        info!("{json:?}");

        // Done with this entity
        commands.entity(e).despawn_recursive();
    }
}
