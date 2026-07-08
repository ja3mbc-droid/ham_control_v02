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

pub fn get_vfo() -> Result<String, String> {
    call_method("rig.get_vfo")
}

pub fn get_mode() -> Result<String, String> {
    call_method("rig.get_mode")
}

pub fn get_ptt() -> Result<String, String> {
    call_method("rig.get_ptt")
}
