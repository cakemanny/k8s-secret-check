use futures::{StreamExt, TryStreamExt};
use kube_client::api::{Api, ResourceExt, ListParams, WatchEvent};
use kube_client::Client;
use k8s_openapi::api::core::v1::Secret;
use yaml_rust::{YamlLoader};
use serde_json::Value;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the environment to find config for kube client.
    // Note that this tries an in-cluster configuration first,
    // then falls back on a kubeconfig file.
    let client = Client::try_default().await?;

    let secrets: Api<Secret> = Api::default_namespaced(client);

    let mut stream  = secrets.watch(&ListParams::default(), "0").await?.boxed();

    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(s) => {
                println!("Added {}, validating...", s.name());
                validate_secret(s);
            },
            WatchEvent::Modified(s) => {
                println!("Modified: {}, validating...", s.name());
                validate_secret(s)
            },
            WatchEvent::Deleted(_s) => {},
            WatchEvent::Bookmark(_s) => {},
            WatchEvent::Error(s) => println!("{}", s),
        }
    }

    Ok(())
}

fn validate_secret(s: Secret) {
    let secret_name = s.name();

    if let Some(data) = s.data {
        for k in data.keys() {
            if k.ends_with(".yaml") {
                let secret_bytes = &data[k];
                match validate_yaml(secret_bytes) {
                    Err(e) => {
                        println!("{}, key {} contains invalid YAML: {}", secret_name, k, e)
                    }
                    Ok(()) => { /* be silent */ }
                }
            } else if k.ends_with(".json") {
                let secret_bytes = &data[k];
                match validate_json(secret_bytes) {
                    Err(e) => {
                        println!("{}, key {} contains invalid JSON: {}", secret_name, k, e)
                    }
                    Ok(()) => {
                        println!("{}, key {} is valid", secret_name, k)
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
