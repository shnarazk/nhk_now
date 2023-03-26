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
    let json = use_future(cx, (count,), |(count,)| async move {
        load_json(&app_config).await
    });
    // dbg!(json);
    match json.value() {
        Some(Ok(json)) => {
            // dbg!(&json["nowonair_list"]["g1"]["present"]);
            // dbg!(&json["nowonair_list"]["g1"]);
            cx.render(rsx!{
                div {
                    class: "tabs mt-2 ml-2",
                    // class: "btn-group ml-4 mt-4",
                    button {
                        class: "tab tab-lifted tab-active text-lg",
                        // class: "btn bg-secondary focus:bg-secondary no-animation",
                        "NHK総合プログラム"
                    }
                    button {
                        class: "tab tab-lifted text-lg",
                        // class: "btn bg-secondary no-animation",
                        "Eテレ"
                    }
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
                                    class: "btn btn-ghost btn-outline btn-sm",
                                    onclick: move |_| { count += 1 },
                                    "プログラム更新"
                                }
                            }
                        }
                        tr {
                            class:"bg-slate-100 text-gray-600",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["nowonair_list"]["g1"]["following"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["nowonair_list"]["g1"]["following"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-100 text-gray-600",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["nowonair_list"]["g1"]["following"]["subtitle"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-200 text-black",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["nowonair_list"]["g1"]["present"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["nowonair_list"]["g1"]["present"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-200 text-black",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["nowonair_list"]["g1"]["present"]["subtitle"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-400 text-gray-800",
                            td {
                                class:"",
                                DateTime::parse_from_rfc3339(json["nowonair_list"]["g1"]["previous"]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                            }
                            td {
                                class:"",
                                json["nowonair_list"]["g1"]["previous"]["title"].as_str(),
                            }
                        }
                        tr {
                            class:"bg-slate-400 text-gray-800",
                            td {
                                colspan: 2,
                                class:"whitespace-normal pl-8 w-4/5 text-sm",
                                json["nowonair_list"]["g1"]["previous"]["subtitle"].as_str(),
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
                    "..."
            }
        })),
    }
}

async fn load_json(config: &AppConfig) -> hyper::Result<Value> {
    let area = config.area.as_deref().unwrap_or("400");
    let service = config.service.as_deref().unwrap_or("g1");
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
