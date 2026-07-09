use reqwest::blocking::Client;

fn call_method(method: &str) -> Result<String, String> {
    let xml = format!(
        r#"<?xml version="1.0"?>
<methodCall>
<methodName>{}</methodName>
<params/>
</methodCall>"#,
        method
    );

    let client = Client::new();

    let response = client
        .post("http://127.0.0.1:12345")
        .header("Content-Type", "text/xml")
        .body(xml)
        .send()
        .map_err(|e| e.to_string())?;

    response.text().map_err(|e| e.to_string())
}

fn extract_value(xml: &str) -> String {
    let inner = xml
        .split("<value>")
        .nth(1)
        .and_then(|s| s.split("</value>").next())
        .unwrap_or("0")
        .trim()
        .to_string();

    // <i4>123</i4> のような内側タグも剥がす
    if inner.starts_with('<') {
        if let Some(gt_pos) = inner.find('>') {
            let after_open = &inner[gt_pos + 1..];
            if let Some(lt_pos) = after_open.find('<') {
                return after_open[..lt_pos].trim().to_string();
            }
        }
    }

    inner
}

/// 周波数(Hz、文字列)を取得
pub fn get_vfo() -> Result<String, String> {
    call_method("rig.get_vfo").map(|xml| extract_value(&xml))
}

/// モード(例: "LSB", "USB")を取得
pub fn get_mode() -> Result<String, String> {
    call_method("rig.get_mode").map(|xml| extract_value(&xml))
}

/// PTT状態を取得(true=送信中, false=受信中)
pub fn get_ptt() -> Result<bool, String> {
    call_method("rig.get_ptt").map(|xml| extract_value(&xml) == "1")
}
