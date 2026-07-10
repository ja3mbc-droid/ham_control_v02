use reqwest::blocking::Client;

fn call_method(addr: &str, method: &str) -> Result<String, String> {
    let xml = format!(
        r#"<?xml version="1.0"?>
<methodCall>
<methodName>{}</methodName>
<params/>
</methodCall>"#,
        method
    );

    let client = Client::new();
    let url = format!("http://{}", addr);

    let response = client
        .post(&url)
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

pub fn get_vfo(addr: &str) -> Result<String, String> {
    call_method(addr, "rig.get_vfo").map(|xml| extract_value(&xml))
}

pub fn get_mode(addr: &str) -> Result<String, String> {
    call_method(addr, "rig.get_mode").map(|xml| extract_value(&xml))
}

pub fn get_ptt(addr: &str) -> Result<bool, String> {
    call_method(addr, "rig.get_ptt").map(|xml| extract_value(&xml) == "1")
}

/// Sメータ値(生の文字列。単位・スケールはリグ機種依存)を取得
pub fn get_smeter(addr: &str) -> Result<String, String> {
    call_method(addr, "rig.get_smeter").map(|xml| extract_value(&xml))
}
