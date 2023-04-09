use {
    bevy::{
        prelude::*,
        tasks::{AsyncComputeTaskPool, Task},
    },
    // chrono::prelude::*,
    // chrono::DateTime,
    clap::Parser,
    hyper::Client,
    hyper_tls::HttpsConnector,
    serde_json::Value,
};

const ACTIVE_CHANNEL_COLOR: Color = Color::rgb(1., 0.066, 0.349);
const JUSTIFY_CONTENT_COLOR: Color = Color::rgb(0.102, 0.522, 1.);
const MARGIN: Val = Val::Px(5.);

#[derive(Clone, Component, Debug, Default, Parser)]
#[clap(author, version, about)]
struct AppConfig {
    /// area code
    #[clap(short = 'a')]
    area: Option<String>,
    /// service (channel)
    #[clap(short = 's')]
    service: Option<String>,
    /// date
    #[clap(short = 'd')]
    date: Option<String>,
    /// API key
    #[clap(short = 'k', long = "key", env)]
    apikey: String,
    /// Just download the csv w/o GUI
    #[clap(long = "headless")]
    headless: bool,
}

#[allow(dead_code)]
const SERVICES: [(&str, &str); 5] = [
    ("g1", "NHK総合1"),
    ("e1", "NHKEテレ1"),
    ("r1", "NHKラジオ第1"),
    ("r2", "NHKラジオ第2"),
    ("r3", "NHKFM"),
];
#[allow(dead_code)]
const PROGRAMS: [(&str, &str); 3] = [
    ("following", "bg-slate-100 text-gray-600"),
    ("present", "bg-slate-200 text-black"),
    ("previous", "bg-slate-400 text-gray-800"),
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: [700., 440.].into(),
                title: "NHK now".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, (spawn_layout, spawn_tasks))
        .add_systems(Update, (button_system, handle_tasks))
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
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        ACTIVE_CHANNEL_COLOR,
                        UiRect::right(MARGIN),
                        "NHK総合",
                    );
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        "Eテレ",
                    );
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        "ラジオ第1",
                    );
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        "ラジオ第2",
                    );
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        "NHK FM",
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
                .with_children(|builder| {
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
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with_children(|builder| {
                            spawn_child_node(
                                builder,
                                font.clone(),
                                AlignItems::Baseline,
                                JustifyContent::Center,
                            );
                        });
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::width(Val::Percent(100.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(ButtonBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                        // horizontally center child text
                                        justify_content: JustifyContent::Center,
                                        // vertically center child text
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        "button",
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: 40.0,
                                            color: Color::rgb(0.9, 0.9, 0.9),
                                        },
                                    ));
                                });
                        });
                });
        });
}

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

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
type ButtonLike = (Changed<Interaction>, With<Button>);

fn button_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &Children), ButtonLike>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "更新中".to_string();
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                text.sections[0].value = "更新".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "プログラム".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

#[derive(Component)]
struct ProgramJson(Task<Value>);

fn spawn_tasks(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let config = AppConfig::default();
        let Ok(json) = load_json(&config, "g1").await else {
            return serde_json::from_str("{status: \"no data\"}").unwrap();
        };
        json
    });
    commands.spawn(ProgramJson(task));
}

fn handle_tasks(mut _commands: Commands, mut _tasks: Query<(Entity, &mut ProgramJson)>) {
    // TODO:
}

#[allow(dead_code)]
async fn load_json(config: &AppConfig, service: &str) -> hyper::Result<Value> {
    let area = config.area.as_deref().unwrap_or("400");
    let key = &config.apikey;
    // "https://api.nhk.or.jp/v2/pg/list/{area}/{service}/{date}.json?key={key}"
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let base = format!("https://api.nhk.or.jp/v2/pg/now/{area}/{service}.json?key={key}")
        .parse()
        .expect("wrong url");
    dbg!(&base);
    let res = client.get(base).await?;
    dbg!();
    let buf = hyper::body::to_bytes(res).await?;
    let str = String::from_utf8_lossy(buf.as_ref());
    // dbg!(&str);
    let json: Value = serde_json::from_str(str.to_string().as_str()).expect("invalid json");
    // dbg!(&json);
    Ok(json)
}
