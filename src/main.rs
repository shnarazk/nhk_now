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
            .with_custom_head("<link href=\"https://cdn.jsdelivr.net/npm/daisyui@2.51.5/dist/full.css\" rel=\"stylesheet\" type=\"text/css\" />\n<script src=\"https://cdn.tailwindcss.com\"></script>".to_string()),
    );
}

#[allow(unused_variables)]
fn app(cx: Scope) -> Element {
    let app_config = AppConfig::parse();
    let mut count = use_state(cx, || 0);
    let s = match &app_config.service {
        Some(ref s) => s.clone(),
        _ => "g1".to_string(),
    };
    let service = use_state(cx, || s);
    let json = use_future(cx, (count, service), |(count, service)| async move {
        load_json(&app_config, &*service).await
    });
    // dbg!(json);
    macro_rules! TAB_CLASS {
        ($target: expr) => {
            if $target == service.as_str() {
                "tab tab-lifted text-lg tab-active"
            } else {
                "tab tab-lifted text-lg"
            }
        };
    }
    match json.value() {
        Some(Ok(json)) if !json["nowonair_list"][service.as_str()].is_null() => {
            // dbg!(&json["nowonair_list"][service.as_str()]["following"]["start_time"]);
            cx.render(rsx!{
                div {
                    class: "tabs mt-2 ml-2",
                    button {
                        class: TAB_CLASS!("g1"),
                        onclick: move |_| service.set("g1".to_string()),
                        "NHK総合1"
                    },
                    button {
                        class: TAB_CLASS!("e1"),
                        onclick: move |_| service.set("e1".to_string()),
                        "NHKEテレ1"
                    },
                    button {
                        class: TAB_CLASS!("r1"),
                        onclick: move |_| service.set("r1".to_string()),
                        "NHKラジオ第1"
                    },
                    button {
                        class: TAB_CLASS!("r2"),
                        onclick: move |_| service.set("r2".to_string()),
                        "NHKラジオ第2"
                    },
                    button {
                        class: TAB_CLASS!("r3"),
                        onclick: move |_| service.set("r3".to_string()),
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
                                    "プログラム更新"
                                }
                            }
                        }
                        tr {
                            class:"bg-slate-100 text-gray-600",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["nowonair_list"][service.as_str()]["following"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["nowonair_list"][service.as_str()]["following"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-100 text-gray-600",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["nowonair_list"][service.as_str()]["following"]["subtitle"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-200 text-black",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["nowonair_list"][service.as_str()]["present"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["nowonair_list"][service.as_str()]["present"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-200 text-black",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["nowonair_list"][service.as_str()]["present"]["subtitle"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-400 text-gray-800",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["nowonair_list"][service.as_str()]["previous"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["nowonair_list"][service.as_str()]["previous"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-400 text-gray-800",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["nowonair_list"][service.as_str()]["previous"]["subtitle"].as_str(),
                            }
                        }
                    }
                }
            })
        }
        _ => cx.render(rsx!(
        div {
            class: "grid grid-cols-1 place-items-center h-[300px]",
            div {
                class: "radial-progress animate-spin w-20 h-20",
                style: "--value:70;",
                ""
            }
        })),
    }
}

async fn load_json(config: &AppConfig, service: &str) -> hyper::Result<Value> {
    let area = config.area.as_deref().unwrap_or("400");
    let key = &config.apikey;
    // "https://api.nhk.or.jp/v2/pg/list/{area}/{service}/{date}.json?key={key}"
    let base = format!("https://api.nhk.or.jp/v2/pg/now/{area}/{service}.json?key={key}");
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let res = client.get(base.parse().expect("wrong url")).await?;
    let buf = hyper::body::to_bytes(res).await?;
    let str = String::from_utf8_lossy(buf.as_ref());
    let json: Value = serde_json::from_str(str.to_string().as_str()).expect("invalid json");
    // dbg!(&json);
    Ok(json)
}
