use {
    chrono::DateTime, clap::Parser, dioxus::prelude::*, dioxus_desktop::Config, serde_json::Value,
};

const SERVICES: [(&str, &str); 5] = [
    ("g1", "NHK総合1"),
    ("e1", "NHKEテレ1"),
    ("r1", "NHKラジオ第1"),
    ("r2", "NHKラジオ第2"),
    ("r3", "NHKFM"),
];
const PROGRAMS: [(&str, &str); 3] = [
    ("following", "bg-slate-100 text-gray-600"),
    ("present", "bg-slate-200 text-black"),
    ("previous", "bg-slate-400 text-gray-800"),
];

#[derive(Clone, Debug, Default, Eq, Parser, PartialEq, Props)]
#[clap(author, version, about)]
struct AppConfig {
    /// area code
    #[clap(short = 'a', default_value = "400")]
    area: u32,
    /// API key
    #[clap(short = 'k', long = "key", env)]
    apikey: String,
}

#[tokio::main]
async fn main() {
    let app_config = AppConfig::parse();
    dioxus_desktop::launch_with_props(
        app,
        app_config,
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

fn app(cx: Scope<AppConfig>) -> Element {
    let json: &UseState<Value> = use_state(cx, || serde_json::from_str("{\"NO\": {}}").unwrap());
    macro_rules! Fetch {
        ($service: expr) => {{
            |_| {
                cx.spawn({
                    let conf = cx.props.clone();
                    let ch = $service.to_string();
                    let json = json.to_owned();
                    async move {
                        eprintln!("open");
                        if let Ok(data) = fetch_json_reqwest(conf, ch).await {
                            json.set(data);
                        }
                        eprintln!("end");
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
    match parse_json(json.get()) {
        Some((data, ch)) if !ch.is_empty() => {
            let refetch = {
                let c = cx.props.clone();
                let ch = ch.to_string();
                move |_| {
                    cx.spawn({
                        let c = c.clone();
                        let ch = ch.clone();
                        let js = json.to_owned();
                        async move {
                            if let Ok(resp) = fetch_json_reqwest(c, ch).await {
                                js.set(resp);
                            }
                        }
                    })
                }
            };
            cx.render(rsx!(
                div {
                    class: "tabs mt-2 ml-2",
                    for s in SERVICES.iter() {
                        button { class: TAB_CLASS!(s.0, ch), onclick: Fetch!(s.0), "{s.1}"}
                    }
                },
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
                        for p in PROGRAMS.iter() {
                            tr {
                                class: p.1,
                                td {
                                    DateTime::parse_from_rfc3339(data[p.0]["start_time"].as_str().unwrap()).unwrap().format("%H:%M").to_string(),
                                }
                                td {
                                    data[p.0]["title"].as_str(),
                                }
                            }
                            tr {
                                class: p.1,
                                td {
                                    colspan: 2,
                                    class: "whitespace-normal pl-8 w-4/5 text-sm",
                                    data[p.0]["subtitle"].as_str(),
                                }
                            }
                        }
                    }
                })
            )
        }
        Some(_) => cx.render(rsx!(
            div {
                class: "tabs mt-2 ml-2",
                for s in SERVICES.iter() {
                    button { class: TAB_CLASS!(s.0, ""), onclick: Fetch!(s.0), "{s.1}"}
                }
            },
            div {
                class: "grid grid-cols-1 place-items-center h-[300px]",
                "サービスを選んで下さい"
            }
        )),
        _ => cx.render(rsx!(
            div {
                class: "tabs mt-2 ml-2",
                for s in SERVICES.iter() {
                    button { class: TAB_CLASS!(s.0, ""), onclick: Fetch!(s.0), "{s.1}"}
                }
            },
            div {
                class: "grid grid-cols-1 place-items-center h-[300px]",
                div {
                    class: "radial-progress animate-spin w-20 h-20",
                    style: "--value:70;",
                    ""
                }
            }
        )),
    }
}

async fn fetch_json_reqwest(config: AppConfig, service: String) -> Result<Value, ()> {
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
