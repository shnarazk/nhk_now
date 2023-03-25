use {
    clap::Parser, dioxus::prelude::*, dioxus_desktop::Config, hyper::Client,
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
    #[clap(short = 'k', long)]
    key: String,
    /// Just download the csv w/o GUI
    #[clap(long = "headless")]
    headless: bool,
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    dioxus_desktop::launch_cfg(
        app,
        Config::new()
            .with_custom_head("<script src=\"https://cdn.tailwindcss.com\"></script>".to_string()),
    );
}

fn app(cx: Scope) -> Element {
    let app_config = AppConfig::parse();
    let json = use_future(cx, (), |_| async move { load_json(&app_config).await }).value();
    dbg!(json);
    match json {
        Some(Ok(json)) => {
            dbg!(&json["nowonair_list"]["g1"]["present"]["title"]);
            cx.render(rsx!{
                h1 {
                    class: "inline-block bg-slate-200 m-4 text-lg text-red-600 drop-shadow-xl border-solid border-2 border-indigo-600 rounded",
                    "NHK綜合プログラム"
                }
                ul {
                    li {
                        class: "inline-block bg-slate-200 m-10 text-slate-600 drop-shadow-xl border-solid border-0 border-indigo-600 rounded",
                        format!("{}", json["nowonair_list"]["g1"]["present"]["title"]),
                    }
                }
            })
        }
        _ => cx.render(rsx!(h1 { "..."})),
    }
}

async fn load_json(config: &AppConfig) -> hyper::Result<Value> {
    dbg!();
    let area = config.area.as_deref().unwrap_or("400");
    let service = config.service.as_deref().unwrap_or("g1");
    let key = &config.key;
    // "https://api.nhk.or.jp/v2/pg/list/{area}/{service}/{date}.json?key={key}"
    let base = format!("https://api.nhk.or.jp/v2/pg/now/{area}/{service}.json?key={key}");
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    dbg!();
    let res = client.get(base.parse().expect("wrong url")).await?;
    let buf = hyper::body::to_bytes(res).await?;
    dbg!();
    let str = String::from_utf8_lossy(buf.as_ref());
    let json: Value = serde_json::from_str(str.to_string().as_str()).expect("invalid json");
    dbg!(&json);
    Ok(json)
}
