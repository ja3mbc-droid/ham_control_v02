use reqwest::blocking::Client;

pub fn get_vfo() -> Result<String, String> {
    let xml = r#"<?xml version="1.0"?>
<methodCall>
<methodName>rig.get_vfo</methodName>
<params/>
</methodCall>"#;

    let client = Client::new();

    let response = client
        .post("http://127.0.0.1:12345")
        .header("Content-Type", "text/xml")
        .body(xml)
        .send()
        .map_err(|e| e.to_string())?;

    response.text().map_err(|e| e.to_string())
}
