use const_format::concatcp;

pub const APP: &str = "service-demo";
pub const BASE_PATH: &str = concatcp!("/", APP, "/v1");
