use {
    chrono::prelude::*, clap::Parser, dioxus::prelude::*, dioxus_desktop::Config, hyper::Client,
    hyper_tls::HttpsConnector, serde_json::Value,
};

#[derive(Clone, Debug, Default, Parser)]
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

#[allow(unused_variables)]
fn app(cx: Scope) -> Element {
    let app_config = AppConfig::parse();
    let mut count = use_state(cx, || 0);
    let channel = use_state(cx, || match &app_config.service {
        Some(s) => s.clone(),
        _ => "g1".to_string(),
    });
    let json = {
        let c = app_config.clone();
        use_future(cx, (count, channel), |(count, channel)| async move {
            load_json_reqwest(c, channel.get().clone()).await
        })
    };
    macro_rules! TAB_CLASS {
        ($target: expr, $service: expr) => {
            if $target == $service {
                "tab tab-lifted text-lg tab-active"
            } else {
                "tab tab-lifted text-lg"
            }
        };
    }
    match parse_json(json.value()) {
        Some((json, ch)) => {
            cx.render(rsx!{
                div {
                    class: "tabs mt-2 ml-2",
                    button {
                        class: TAB_CLASS!("g1", ""),
                        onclick: move |_| channel.set("g1".to_string()),
                        "NHK総合1"
                    },
                    button {
                        class: TAB_CLASS!("e1", ""),
                        onclick: move |_| channel.set("e1".to_string()),
                        "NHKEテレ1"
                    },
                    button {
                        class: TAB_CLASS!("r1", ""),
                        onclick: move |_| channel.set("r1".to_string()),
                        "NHKラジオ第1"
                    },
                    button {
                        class: TAB_CLASS!("r2", ""),
                        onclick: move |_| channel.set("r2".to_string()),
                        "NHKラジオ第2"
                    },
                    button {
                        class: TAB_CLASS!("r3", ""),
                        onclick: move |_| channel.set("r3".to_string()),
                        "NHKFM"
                    },
                }
                div {
                    class: "grid bg-base-300 p-0 mx-2 drop-shadow-xl",
                    table {
                        class:"table table-compact p-0 mt-0 w-full text-white bg-red-600 border-red-600 border-y-2 border-solid border-0 border-indigo-600",
                        tr {
                            class:"",
                            th {
                                "開始時間"
                            }
                            th {
                                class:"text-right",
                                button {
                                    class: "btn btn-outline btn-sm text-neutral-200",
                                    onclick: move |_| { count += 1 },
                                    "表示情報更新"
                                }
                            }
                        }
                        tr {
                            class:"bg-slate-100 text-gray-600",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["following"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["following"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-100 text-gray-600",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["following"]["subtitle"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-200 text-black",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["present"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["present"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-200 text-black",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["present"]["subtitle"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-400 text-gray-800",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["previous"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["previous"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-400 text-gray-800",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["previous"]["subtitle"].as_str(),
                            }
                        }
                    }
                }
            })
        }
        _ => {
            dbg!();
            cx.render(rsx! {
                div {
                    class: "tabs mt-2 ml-2",
                    button {
                        class: TAB_CLASS!("g1", ""),
                        onclick: move |_| channel.set("g1".to_string()),
                        "NHK総合1"
                    },
                    button {
                        class: TAB_CLASS!("e1", ""),
                        onclick: move |_| channel.set("e1".to_string()),
                        "NHKEテレ1"
                    },
                    button {
                        class: TAB_CLASS!("r1", ""),
                        onclick: move |_| channel.set("r1".to_string()),
                        "NHKラジオ第1"
                    },
                    button {
                        class: TAB_CLASS!("r2", ""),
                        onclick: move |_| channel.set("r2".to_string()),
                        "NHKラジオ第2"
                    },
                    button {
                        class: TAB_CLASS!("r3", ""),
                        onclick: move |_| channel.set("r3".to_string()),
                        "NHKFM"
                    },
                }
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
            })
        }
    }
}

#[allow(dead_code)]
async fn load_json(config: &AppConfig, service: &str) -> hyper::Result<Value> {
    let client = Client::builder()
        .retry_canceled_requests(true)
        .build::<_, hyper::Body>(HttpsConnector::new());
    let base = {
        // "https://api.nhk.or.jp/v2/pg/list/{area}/{service}/{date}.json?key={key}"
        let area = config.area.as_deref().unwrap_or("400");
        let key = &config.apikey;
        format!("https://api.nhk.or.jp/v2/pg/now/{area}/{service}.json?key={key}")
            .parse()
            .expect("wrong url")
    };
    // dbg!();
    let res = client.get(base).await?;
    // dbg!(&res);
    let buf = hyper::body::to_bytes(res).await?;
    let str = String::from_utf8_lossy(buf.as_ref());
    assert!(!str.is_empty());
    // dbg!(&str);
    let json: Value = serde_json::from_str(str.to_string().as_str()).expect("invalid json");
    // dbg!(&json);
    Ok(json)
}

async fn load_json_reqwest(config: AppConfig, service: String) -> hyper::Result<Value> {
    let base = {
        // "https://api.nhk.or.jp/v2/pg/list/{area}/{service}/{date}.json?key={key}"
        let area = config.area.as_deref().unwrap_or("400");
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

fn parse_json(json: Option<&hyper::Result<Value>>) -> Option<(Value, String)> {
    let Some(Ok(json)) = json else {
        return None;
    };
    for ch in ["g1", "e1", "r1", "r2", "r3"] {
        if let Some(j) = json["nowonair_list"].get(ch) {
            dbg!(&j);
            return Some((j.clone(), ch.to_string()));
        }
    }
    None
}
