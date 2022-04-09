use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Secret;
use kube_client::Client;
use kube_client::api::{Api, ResourceExt, ListParams, WatchEvent};
use serde::Serialize;
use serde_json::Value;
use std::env;
use yaml_rust::{YamlLoader};
use reqwest;


#[derive(Serialize)]
pub enum LogSeverity {
    DEBUG, INFO, NOTICE, WARNING, ERROR
}

#[derive(Serialize)]
pub struct LogEntry {
    severity: LogSeverity,
    message: String,
    // I think we could add secret name to this to be more structural
}

fn log(severity: LogSeverity, message: String) {
    let entry = LogEntry{ severity, message };
    // Yes, this seems a bit wrong
    println!("{}", serde_json::to_string(&entry).unwrap());
}

fn log_debug(msg: String) { log(LogSeverity::DEBUG, msg) }
fn log_notice(msg: String) { log(LogSeverity::NOTICE, msg) }
fn log_err(msg: String) { log(LogSeverity::ERROR, msg) }

#[derive(Serialize)]
pub struct WebhookBody {
    text: String,
}

async fn notify(msg: String) {

    if let Ok(webhook_url) = env::var("SLACK_WEBHOOK_URL") {
        let body = WebhookBody{text: msg};
        let client = reqwest::Client::new();
        let res_or_err = client.post(webhook_url)
            .json(&body)
            .send()
            .await;

        match res_or_err {
            Err(err) => {
                log_err(format!("{}", err))
            },
            Ok(res) => {
                log_debug(format!("Webhook fired: {}", res.status()));
            }
        }
    }
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_notice("k8s-secret-check started".to_string());

    if env::var("SLACK_WEBHOOK_URL").is_err() {
        log_notice("SLACK_WEBHOOK_URL is not configured.".to_string());
    }

    // Read the environment to find config for kube client.
    // Note that this tries an in-cluster configuration first,
    // then falls back on a kubeconfig file.
    let client = Client::try_default().await?;

    let secrets: Api<Secret> = Api::default_namespaced(client);

    let mut resource_version = "0".to_owned();
    loop {
        let mut stream  = secrets.watch(&ListParams::default(), &resource_version).await?.boxed();

        while let Some(event) = stream.try_next().await? {
            match event {
                WatchEvent::Added(s) => {
                    log_debug(format!("{} added, validating...", s.name()));
                    validate_secret(s).await;
                },
                WatchEvent::Modified(s) => {
                    log_debug(format!("{} modified, validating...", s.name()));
                    validate_secret(s).await;
                },
                WatchEvent::Deleted(_s) => {},
                WatchEvent::Bookmark(bm) => {
                    log_debug(format!("bookmark: {}", bm.metadata.resource_version));
                    resource_version = bm.metadata.resource_version;
                },
                WatchEvent::Error(s) => log_err(format!("{}", s)),
            }
        }

        let secret_list = secrets.list(&ListParams::default()).await?;
        if let Some(rv) = secret_list.metadata.resource_version {
            log_debug(format!("new res.ver: {}", rv));
            resource_version = rv;
        }
    }
}

async fn validate_secret(s: Secret) {
    let secret_name = s.name();

    if let Some(data) = s.data {
        for k in data.keys() {
            if k.ends_with(".yaml") {
                let secret_bytes = &data[k];
                match validate_yaml(secret_bytes) {
                    Err(e) => {
                        log_err(format!("{}, key {} contains invalid YAML: {}", secret_name, k, e));
                        notify(format!("{}, key {} contains invalid YAML: {}", secret_name, k, e)).await;
                    }
                    Ok(()) => {
                        log_debug(format!("{}, key {} is valid", secret_name, k))
                    }
                }
            } else if k.ends_with(".json") {
                let secret_bytes = &data[k];
                match validate_json(secret_bytes) {
                    Err(e) => {
                        log_err(format!("{}, key {} contains invalid JSON: {}", secret_name, k, e));
                        notify(format!("{}, key {} contains invalid JSON: {}", secret_name, k, e)).await;
                    }
                    Ok(()) => {
                        log_debug(format!("{}, key {} is valid", secret_name, k))
                    }
                }
            }
        }
    }
}

fn validate_yaml(secret_data: &k8s_openapi::ByteString) -> Result<(), Box<dyn std::error::Error>> {
    let k8s_openapi::ByteString(vec) = secret_data;
    let secret_str = std::str::from_utf8(vec)?;
    YamlLoader::load_from_str(secret_str)?;
    Ok(())
}

fn validate_json(secret_data: &k8s_openapi::ByteString) -> Result<(), Box<dyn std::error::Error>> {
    let k8s_openapi::ByteString(secret_bytes) = secret_data;
    let _: Value = serde_json::from_slice(secret_bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_yaml() {
        let s =
"
foo:
    - list1
    - list2
bar:
    - 1
    - 2.0
";
        // unwrap to crash
        let input = k8s_openapi::ByteString(s.as_bytes().to_vec());
        validate_yaml(&input).unwrap();
    }

    #[test]
    fn invalid_yaml() {
        let s =
"
foo: list1
    mistaken: indentation
";
        let input = k8s_openapi::ByteString(s.as_bytes().to_vec());
        let result = validate_yaml(&input);
        assert!(result.is_err());
    }

    #[test]
    fn valid_json() {
        let s = r#"{"vali": "djson"}"#;
        // unwrap to crash
        let input = k8s_openapi::ByteString(s.as_bytes().to_vec());
        validate_yaml(&input).unwrap();
    }

    #[test]
    fn invalid_json() {
        let s = r#"{"test" "forgot colon"}"#;
        let input = k8s_openapi::ByteString(s.as_bytes().to_vec());
        let result = validate_yaml(&input);
        assert!(result.is_err());
    }

}
