use std::str;
use curl::easy::Easy;
use std::collections::HashMap;
use std::io::Read;
use serde_json::Value;
use gato_core::kernel::Logger;

pub struct LambdaDriver {
    headers: HashMap<String, String>,
    body_raw: String,
    body: Value
}

impl LambdaDriver {
    pub fn new() -> Self {
        let mut driver = LambdaDriver { body_raw: String::new(), body: Value::Null, headers: HashMap::new() };
        driver.get_payload();
        return driver;
    }

    fn get_payload(&mut self) {
        let endpoint = std::env::var("AWS_LAMBDA_RUNTIME_API")
            .unwrap_or("127.0.0.1:9001".to_owned());
        let url = format!("http://{}/2018-06-01/runtime/invocation/next", endpoint);

        let mut data = Vec::new();
        let mut easy = Easy::new();
        easy.url(url.as_str()).unwrap();
        {
            let mut transfer = easy.transfer();
            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            }).unwrap();
            transfer.header_function(|new_data| {
                let h = str::from_utf8(&new_data).unwrap_or("");
                if h.trim().len() > 0 {
                    let index = h.find(":").unwrap_or(h.len());

                    let name = h.get(0..index).unwrap_or("");
                    let value = h.get(index + 1..h.len()).unwrap_or("");

                    if value.len() > 0 {
                        self.headers.insert(name.trim().to_string(), value.trim().to_string());
                    }
                }
                return true;
            }).unwrap();
            match transfer.perform() {
                Ok(_) => {
                    Logger::info(format!("[lambda] request config success").as_str());
                },
                Err(error) => {
                    Logger::error(format!("[lambda] error requesting lambda {}", error).as_str());
                }
            }
        }
        self.body_raw = format!("{}", str::from_utf8(&data).unwrap_or("")).as_str().to_string();
        self.body = serde_json::from_str(self.body_raw.as_str()).unwrap_or(Value::Null);
    }

    pub fn get_method(&self) -> String {
        return self.body["httpMethod"].as_str().unwrap_or("GET").to_string();
    }

    pub fn get_uri(&self) -> String {
        return self.body["path"].as_str().unwrap_or("/").to_string();
    }

    pub fn get_headers(&self) -> HashMap<String, String> {
        let headers = self.body["headers"].as_object().unwrap();
        let mut h: HashMap<String, String> = HashMap::new();
        for (name, value) in headers {
            h.insert(name.to_string(), value.as_str().unwrap_or("").to_string());
        }
        return h;
    }

    pub fn get_body(&self) -> String {
        let body_is_b64 = self.body["isBase64Encoded"].as_bool().unwrap_or(false);
        let body: String;
        if self.body["body"].is_string() {
            body = self.body["body"].as_str().unwrap().to_string();
        } else {
            body = self.body["body"].to_string();
        }
        if body_is_b64 {
            return match base64::decode(body.as_str()) {
                Ok(r) => str::from_utf8(&r).unwrap_or("").to_string(),
                Err(_e) => body
            };
        }
        return body;
    }

    pub fn send(&self, data: String) {
        let endpoint = std::env::var("AWS_LAMBDA_RUNTIME_API")
            .unwrap_or("127.0.0.1:9001".to_owned());
        let id = self.headers.get("Lambda-Runtime-Aws-Request-Id")
            .cloned().unwrap_or("".to_owned());
        let url = format!("http://{}/2018-06-01/runtime/invocation/{}/response", endpoint, id);

         let mut payload = data.as_bytes();

        let mut easy = Easy::new();
        easy.url(url.as_str()).unwrap();
        easy.post(true).unwrap();
        easy.post_field_size(data.len() as u64).unwrap();

        let mut transfer = easy.transfer();
        transfer.read_function(|buf| {
            Ok(payload.read(buf).unwrap_or(0))
        }).unwrap();
        match transfer.perform() {
            Ok(_) => {
                Logger::info(format!("[lambda] response sent success").as_str());
            },
            Err(error) => {
                Logger::error(format!("[lambda] error sending response {}", error).as_str());
            }
        }
    }
}
