use {
    bevy::{
        prelude::*,
        // tasks::{AsyncComputeTaskPool, Task},
        tasks::{IoTaskPool, Task},
        winit::WinitSettings,
    },
    // chrono::DateTime,
    clap::Parser,
    nhk_now::reqwest_plugin::*,
    // hyper::Client,
    // hyper_tls::HttpsConnector,
    serde_json::Value,
};

#[allow(dead_code)]
const SERVICES: [(&str, &str); 5] = [
    ("g1", "NHK総合1"),
    ("e1", "NHKEテレ1"),
    ("r1", "NHKラジオ第1"),
    ("r2", "NHKラジオ第2"),
    ("r3", "NHK FM"),
];
#[allow(dead_code)]
const TIMELINE: [(&str, &str); 3] = [
    ("following", "bg-slate-100 text-gray-600"),
    ("present", "bg-slate-200 text-black"),
    ("previous", "bg-slate-400 text-gray-800"),
];
const ACTIVE_CHANNEL_COLOR: Color = Color::rgb(1., 0.066, 0.349);
const JUSTIFY_CONTENT_COLOR: Color = Color::rgb(0.102, 0.522, 1.);
const MARGIN: Val = Val::Px(2.);

#[derive(Clone, Component, Debug, Default, Eq, PartialEq, Parser)]
#[clap(author, version, about)]
struct AppConfig {
    /// area code
    #[clap(short = 'a', default_value = "400")]
    area: u32,
    /// API key
    #[clap(short = 'k', long = "key", env)]
    apikey: String,
    #[clap(short = 's', long = "service", env)]
    service: String,
}

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
        .add_plugin(ReqwestPlugin)
        // .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ReqestTicket(1))
        .add_systems(Startup, set_config)
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

fn set_config(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn(AppConfig::default());
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
                        "NHK総合1",
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        "NHKEテレ1",
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        "NHKラジオ第1",
                    );
                    spawn_styled_button_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::right(MARGIN),
                        "NHKラジオ第2",
                    );
                    spawn_styled_button_bundle(
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

fn spawn_styled_button_bundle(
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
                .spawn(ButtonBundle {
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
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        text,
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
    mut config_query: Query<&mut AppConfig>,
    mut interaction_query: Query<(&Interaction, &Children), ButtonLike>,
    mut text_query: Query<&mut Text>,
) {
    let mut config = config_query.get_single_mut().unwrap();
    for (interaction, children) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                let text = text_query.get_mut(children[0]).unwrap();
                let label = text.sections[0].value.as_str();
                config.service = label.to_string();

                {
                    let pool = IoTaskPool::get();
                    let config = config.clone();
                    let label = label.to_string();
                    dbg!();
                    let _task = pool.spawn(async move {
                        dbg!();
                        let Ok(json) = fetch_json_reqwest(&config, &label).await else {
                            dbg!();
                        return serde_json::from_str("{status: \"no data\"}").unwrap();
                    };
                        dbg!(json)
                    });
                }
            }
            _ => (),
        }
    }
}

fn button_system2(
    config_query: Query<&AppConfig>,
    mut interaction_query: Query<(&Children, &mut BackgroundColor), With<Button>>,
    mut text_query: Query<&mut Text>,
) {
    let config = config_query.get_single().unwrap();
    let service = config.service.clone();
    for (children, mut color) in &mut interaction_query {
        let text = text_query.get_mut(children[0]).unwrap();
        let label = text.sections[0].value.as_str();
        *color = if service == label {
            ACTIVE_CHANNEL_COLOR.into()
        } else {
            JUSTIFY_CONTENT_COLOR.into()
        };
    }
}

// #[derive(Component)]
// struct ProgramJson(Task<Value>);

// // we need despawn 'task' after reading the content after updating screen.
// fn spawn_tasks(_commands: Commands) {
//     let thread_pool = IoTaskPool::get();
//     let config = AppConfig::default();
//     let service = "g1".to_string();
//     let _task = thread_pool.spawn(async move {
//         let Ok(json) = fetch_json_reqwest(&config, &service).await else {
//             return serde_json::from_str("{status: \"no data\"}").unwrap();
//         };
//         json
//     });
//     // commands.spawn(ProgramJson(task));
// }

// fn handle_tasks(mut _commands: Commands, mut _tasks: Query<(Entity, &mut ProgramJson)>) {
//     // TODO:
// }

/*
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
*/

async fn fetch_json_reqwest(config: &AppConfig, service: &String) -> Result<Value, ()> {
    let base = format!(
        "https://api.nhk.or.jp/v2/pg/now/{}/{}.json?key={}",
        config.area, service, &config.apikey
    );
    println!("1️⃣:build");
    let client = reqwest::Client::builder()
        // .timeout(core::time::Duration::from_secs(8))
        // .connect_timeout(core::time::Duration::from_secs(8))
        // .pool_idle_timeout(core::time::Duration::from_secs(4))
        // .tcp_keepalive(None)
        .build()
        .unwrap();
    println!("2️⃣:send");
    let buf = client
        .get(base)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    println!("3️⃣:received");

    let str = String::from_utf8_lossy(buf.as_ref());
    let json: Value = serde_json::from_str(str.to_string().as_str()).expect("invalid json");
    Ok(json)
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
struct ReqestTicket(u8);

fn send_requests(mut commands: Commands, mut ch: ResMut<ReqestTicket>) {
    if ch.0 == 0 {
        return;
    }
    let Ok(base) = format!(
        "https://api.nhk.or.jp/v2/pg/now/{}/{}.json?key={}",
        400, "g1", "",
    ).as_str().try_into() else {
        return;
    };
    let req = reqwest::Request::new(reqwest::Method::GET, base);
    let req = ReqwestRequest(Some(req));
    ch.0 = 0;
    commands.spawn(req);
    dbg!();
}

fn handle_responses(mut commands: Commands, results: Query<(Entity, &ReqwestBytesResult)>) {
    for (e, res) in results.iter() {
        let string = res.as_str().unwrap();
        info!("{string}");

        // Done with this entity
        commands.entity(e).despawn_recursive();
    }
}
