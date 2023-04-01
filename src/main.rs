use {clap::Parser, serde_json::Value, std::time::Duration};

#[derive(Clone, Debug, Default, Eq, Parser, PartialEq)]
#[clap(author, version, about)]
struct AppConfig {
    /// area code
    #[clap(short = 'a', default_value = "400")]
    area: u16,
    /// API key
    #[clap(short = 'k', long = "key", env)]
    apikey: String,
}

#[tokio::main]
async fn main() {
    let app_config = AppConfig::parse();
    loop {
        std::thread::sleep(Duration::from_secs(10));
        dbg!();
        if let Ok(res) = fetch_json_reqwest(&app_config, "g1".to_string()).await {
            if let Some((_, c)) = parse_json(&res) {
                dbg!(c);
            }
        }
    }
}

async fn fetch_json_reqwest(config: &AppConfig, service: String) -> Result<Value, ()> {
    // "https://api.nhk.or.jp/v2/pg/now/{area}/{service}.json?key={key}"
    let base = format!(
        "https://api.nhk.or.jp/v2/pg/now/{}/{}.json?key={}",
        config.area, service, &config.apikey
    );
    println!("1️⃣build");
    let client = reqwest::Client::builder()
        // .timeout(core::time::Duration::from_secs(8))
        // .connect_timeout(core::time::Duration::from_secs(8))
        // .pool_idle_timeout(core::time::Duration::from_secs(4))
        // .tcp_keepalive(None)
        .build()
        .unwrap();
    println!("2️⃣send");
    let buf = client
        .get(base)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    println!("3️⃣received");

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
