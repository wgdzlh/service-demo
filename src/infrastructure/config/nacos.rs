use std::sync::Arc;

use nacos_sdk::api::{
    config::{ConfigChangeListener, ConfigResponse, ConfigService, ConfigServiceBuilder},
    props::ClientProps,
};

use crate::app::{self, log::*};

const NACOS_ADDR: &str = "0.0.0.0:8848";
const NACOS_USER: &str = "demo";
const NACOS_PASS: &str = "password";
const NACOS_NAMESPACE: &str = "demo-namespace";
const NACOS_DATA_ID: &str = "demo-data-id";
const NACOS_GROUP: &str = "demo-group";

pub fn setup_nacos_conf_sub() -> app::Result<String> {
    let config_service = ConfigServiceBuilder::new(
        ClientProps::new()
            .server_addr(NACOS_ADDR)
            // Attention! "public" is "", it is recommended to customize the namespace with clear meaning.
            .namespace(NACOS_NAMESPACE)
            .app_name(super::APP)
            .auth_username(NACOS_USER)
            .auth_password(NACOS_PASS),
    )
    // .enable_auth_plugin_http()
    .build()?;

    // example get a config
    let config_resp =
        config_service.get_config(NACOS_DATA_ID.to_owned(), NACOS_GROUP.to_owned())?;

    struct DemoConfigChangeListener;

    impl ConfigChangeListener for DemoConfigChangeListener {
        fn notify(&self, config_resp: ConfigResponse) {
            info!("listener get config from nacos: {}", config_resp);
            let cc = super::Config::parse(config_resp.content());
            match cc {
                Ok(conf) => {
                    if let Err(e) = super::set_config(conf, false) {
                        error!("listener set new config failed: {}", e);
                    }
                }
                Err(e) => error!("listener parse new config failed: {}", e),
            }
        }
    }

    // example add a listener
    config_service.add_listener(
        NACOS_DATA_ID.to_owned(),
        NACOS_GROUP.to_owned(),
        Arc::new(DemoConfigChangeListener),
    )?;

    Ok(config_resp.content().clone())
}
