use gato_core::kernel::HttpCoreHandler;
use gato_core::kernel::Provider;
use crate::LambdaHttpCore;

pub struct LambdaServiceProvider { }

impl LambdaServiceProvider {
    pub fn new() -> Box<Self> {
        return Box::new(LambdaServiceProvider {  });
    }
}

impl Provider for LambdaServiceProvider {
    fn boot(&self) {
        let lambda_http_core = LambdaHttpCore::new();
        HttpCoreHandler::set_driver(Box::new(lambda_http_core));
    }
}