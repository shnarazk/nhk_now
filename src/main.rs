use bevy::prelude::*;
// use {chrono::DateTime, clap::Parser, serde_json::Value};

const ACTIVE_CHANNEL_COLOR: Color = Color::rgb(1., 0.066, 0.349);
const JUSTIFY_CONTENT_COLOR: Color = Color::rgb(0.102, 0.522, 1.);
const MARGIN: Val = Val::Px(5.);

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
        .add_systems(Startup, spawn_layout)
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
                        "NHK綜合",
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
                    &text,
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
