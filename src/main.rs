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
    let app_config = AppConfig::parse();
    println!("Hello, world!");
    dbg!(&app_config);
    let Ok(json) = load_json(&app_config).await else { panic!("failed to get a JSON");};
    dbg!(&json["nowonair_list"]["g1"]["present"]["title"]);
    dioxus_desktop::launch_cfg(
        App,
        Config::new()
            .with_custom_head("<script src=\"https://cdn.tailwindcss.com\"></script>".to_string()),
    );
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    cx.render(rsx!(
    h1 {
        class: "inline-block bg-slate-200 m-10 text-red-600 drop-shadow-xl border-solid border-2 border-indigo-600 rounded",
        "NHK綜合プログラム"
    }))
}

async fn load_json(config: &AppConfig) -> hyper::Result<Value> {
    let area = config.area.as_deref().unwrap_or("400");
    let service = config.service.as_deref().unwrap_or("g1");
    let key = &config.key;
    // "https://api.nhk.or.jp/v2/pg/list/{area}/{service}/{date}.json?key={key}"
    let base = format!("https://api.nhk.or.jp/v2/pg/now/{area}/{service}.json?key={key}");
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let res = client.get(base.parse().expect("wrong url")).await?;
    let buf = hyper::body::to_bytes(res).await?;
    let str = String::from_utf8_lossy(buf.as_ref());
    let json: Value = serde_json::from_str(str.to_string().as_str()).expect("invalid json");
    Ok(json)
}
