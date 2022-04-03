
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


