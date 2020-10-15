use serde_json::json;
use std::collections::HashMap;
use crate::lambda_driver::LambdaDriver;
use gato_core::kernel::{HttpCore, RouterHandler, RequestBuilder};

pub struct LambdaHttpCore { }

impl HttpCore for LambdaHttpCore {

    fn handle(&self) {
        loop {
            let driver = LambdaDriver::new();

            // Get RouterHandler Driver
            let router_handler = RouterHandler::get_driver();

            let mut request = RequestBuilder::new();
            request.add_headers(driver.get_headers());
            request.add_body(driver.get_body());
            request.add_method(driver.get_method());
            request.add_uri(driver.get_uri());

            let response = router_handler.handle(&mut request);
            // Send to Lambda
            let body = json!({
                "statusCode": response.get_code(),githu

                "body": response.get_body(),
                "headers": response.get_headers()
            });
            driver.send(body.to_string());
        }
    }

    fn get_request_headers(&self) -> HashMap<String, String> {
        return HashMap::new();
    }

    fn get_post_data(&self) -> String {
        return "".to_owned();
    }
}

impl LambdaHttpCore {
    pub fn new() -> Self {
        return LambdaHttpCore { };
    }
}