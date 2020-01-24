# Rsa keys generator

![](https://github.com/akirill0v/rsa-keys-operator/workflows/CI/badge.svg)

The operator generates private and public RSA keys for the service, mount them in service containers.

## Getting Started

### Installing

Keyman operator can ve installed via Helm(v3) using chart from this repository. To install the chart with release name `keyman`:

```console
git clone https://github.com/akirill0v/rsa-keys-operator.git
helm install keyman ./charts/keyman/ --namespace kube-system
```

## Usage

Just add annotation to deployments for manage keys via operator:

```yaml
# my-deployment.yaml
apiVersion: apps/v1 # for versions before 1.9.0 use apps/v1beta2
kind: Deployment
metadata:
  name: nginx
  annotations:
    "career.evrone.com/service": "true"
spec:
  replicas: 1
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:1.7.9
        ports:
        - containerPort: 80
```
