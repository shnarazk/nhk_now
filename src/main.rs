use {
    chrono::prelude::*, clap::Parser, dioxus::prelude::*, dioxus_desktop::Config, serde_json::Value,
};

#[derive(Clone, Debug, Default, Parser)]
#[clap(author, version, about)]
struct AppConfig {
    /// area code
    #[clap(short = 'a')]
    area: Option<String>,
    // /// service (channel)
    // #[clap(short = 's')]
    // service: Option<String>,
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Props)]
struct AppProps {
    area: String,
    apikey: String,
}

#[tokio::main]
async fn main() {
    // println!("Hello, world!");
    dioxus_desktop::launch_cfg(
        app,
        Config::new()
            .with_custom_head("<link href=\"https://cdn.jsdelivr.net/npm/daisyui@2.51.5/dist/full.css\" rel=\"stylesheet\" type=\"text/css\" />\n<script src=\"https://cdn.tailwindcss.com\"></script>".to_string())
        .with_window(
            dioxus_desktop::tao::window::WindowBuilder::new()
                .with_title("NHK now")
                .with_resizable(true)
                .with_inner_size(dioxus_desktop::wry::application::dpi::LogicalSize::new(640.0,370.0))
            ),
    );
}

fn app(cx: Scope) -> Element {
    let app_config = AppConfig::parse();
    cx.render(rsx! {
        AppBody {
            area: app_config.area.unwrap_or("400".to_string()),
            apikey: app_config.apikey,
        }
    })
}

// #[allow(unused_variables)]
#[allow(non_snake_case)]
fn AppBody(cx: Scope<AppProps>) -> Element {
    let json: &UseState<Value> = use_state(cx, || serde_json::from_str("{\"NO\": {}}").unwrap());
    macro_rules! Fetch {
        ($service: expr) => {{
            let c = cx.props.clone();
            move |_| {
                cx.spawn({
                    let c = c.clone();
                    let ch = $service.to_string();
                    let j = json.to_owned();
                    async move {
                        let resp = load_json_reqwest(c, ch).await;
                        if let Ok(json) = resp {
                            j.set(json);
                        }
                    }
                })
            }
        }};
    }
    macro_rules! TAB_CLASS {
        ($target: expr, $service: expr) => {
            if $target == $service {
                "tab tab-lifted text-lg tab-active"
            } else {
                "tab tab-lifted text-lg"
            }
        };
    }
    cx.render(rsx!{
        div {
            class: "tabs mt-2 ml-2",
            button {
                class: TAB_CLASS!("g1", ""),
                onclick: Fetch!("g1"),
                "NHK総合1"
            },
            button {
                class: TAB_CLASS!("e1", ""),
                onclick: Fetch!("e1"),
                "NHKEテレ1"
            },
            button {
                class: TAB_CLASS!("r1", ""),
                onclick: Fetch!("r1"),
                "NHKラジオ第1"
            },
            button {
                class: TAB_CLASS!("r2", ""),
                onclick: Fetch!("r2"),
                "NHKラジオ第2"
            },
            button {
                class: TAB_CLASS!("r3", ""),
                onclick: Fetch!("r3"),
                "NHKFM"
            },
        },
        match parse_json(json.get()) {
            Some((data, ch)) if !ch.is_empty() => {
                let refetch = {
                    let c = cx.props.clone();
                    move |_| {
                        cx.spawn({
                            let c = c.clone();
                            let ch = ch.to_string();
                            let js = json.to_owned();
                            async move {
                                if let Ok(resp) = load_json_reqwest(c, ch).await {
                                    js.set(resp);
                                }
                            }
                        })
                    }
                };
                rsx!(
                    div {
                        class: "grid bg-base-300 p-0 mx-2 drop-shadow-xl",
                        table {
                            class:"table table-compact p-0 mt-0 w-full text-white bg-red-600 border-red-600 border-y-2 border-solid border-0 border-indigo-600",
                            tr {
                                th { "開始時間" }
                                th {
                                    class: "text-right",
                                    button {
                                        class: "btn btn-outline btn-sm text-neutral-200",
                                        onclick: refetch,
                                        "表示情報更新"
                                    }
                                }
                            }
                            tr {
                                class:"bg-slate-100 text-gray-600",
                                td {
                                    DateTime::parse_from_rfc3339(data["following"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                                }
                                td {
                                    data["following"]["title"].as_str(),
                                }
                            }
                            tr {
                                class:"bg-slate-100 text-gray-600",
                                td {
                                    colspan: 2,
                                    class: "whitespace-normal pl-8 w-4/5 text-sm",
                                    data["following"]["subtitle"].as_str(),
                                }
                            }
                            tr {
                                class:"bg-slate-200 text-black",
                                td {
                                    DateTime::parse_from_rfc3339(data["present"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                                }
                                td {
                                    data["present"]["title"].as_str(),
                                }
                            }
                            tr {
                                class: "bg-slate-200 text-black",
                                td {
                                    colspan: 2,
                                    class:"whitespace-normal pl-8 w-4/5 text-sm",
                                    data["present"]["subtitle"].as_str(),
                                }
                            }
                            tr {
                                class: "bg-slate-400 text-gray-800",
                                td {
                                    DateTime::parse_from_rfc3339(data["previous"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                                }
                                td {
                                    data["previous"]["title"].as_str(),
                                }
                            }
                            tr {
                                class: "bg-slate-400 text-gray-800",
                                td {
                                    colspan: 2,
                                    class:"whitespace-normal pl-8 w-4/5 text-sm",
                                    data["previous"]["subtitle"].as_str(),
                                }
                            }
                        }
                    }
                )
                }
            Some(_) =>
                rsx!(
                    div {
                        class: "grid bg-base-300 p-0 mx-2 drop-shadow-xl",
                        div {
                            class: "grid grid-cols-1 place-items-center h-[300px]",
                            "サービスを選んで下さい"
                        }
                    }
                ),
            _ =>
                rsx!(
                    div {
                        class: "grid bg-base-300 p-0 mx-2 drop-shadow-xl",
                        div {
                            class: "grid grid-cols-1 place-items-center h-[300px]",
                            div {
                                class: "radial-progress animate-spin w-20 h-20",
                                style: "--value:70;",
                                ""
                            }
                        }
                    }
                ),
        }
    })
}

async fn load_json_reqwest(config: AppProps, service: String) -> hyper::Result<Value> {
    let base = {
        // "https://api.nhk.or.jp/v2/pg/list/{area}/{service}/{date}.json?key={key}"
        let area = config.area;
        let key = &config.apikey;
        format!("https://api.nhk.or.jp/v2/pg/now/{area}/{service}.json?key={key}")
    };
    let buf = reqwest::get(base).await.unwrap().bytes().await.unwrap();
    let str = String::from_utf8_lossy(buf.as_ref());
    assert!(!str.is_empty());
    // dbg!(&str);
    let json: Value = serde_json::from_str(str.to_string().as_str()).expect("invalid json");
    // dbg!(&json);
    Ok(json)
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
