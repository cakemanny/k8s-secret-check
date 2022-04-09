
## Idea

You're storing json or yaml in a secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: my-secret
data:
  config.yaml: dGVzdA==
  config.json: eyJ0ZXN0Ijp0cnVlfQ==
```

Since you don't store your secrets in your SCM, you don't have the same
linting and code review processes. Mistakes happen. Things go down.

...

**k8s-secret-check** watches your secret and logs if they are invalid.


## Config

| Env var               | Description |
|-----------------------|-------------|
| `SLACK_WEBHOOK_URL`   | The Slack webhook url for sending notifications to |

## Installation

```shell
cat > kustomization.yaml <<EOF
namespace: secretiveapps
bases:
  - https://github.com/cakemanny/k8s-secret-check/bases/main?ref=master
patches:
- target:
    kind: Deployment
    name: k8s-secret-check
  patch: |-
    apiVersion: apps/v1
    kind: Deployment
    metadata:
      name: k8s-secret-check
    spec:
      template:
        spec:
          containers:
          - name: k8s-secret-check
            env:
            - name: SLACK_WEBHOOK_URL
              value: https://hooks.slack.com/services/XXXXXXXXX/YYYYYYYYYYY/zzzzzzzzzzzzzzzzzzzzzzzz
EOF
```
