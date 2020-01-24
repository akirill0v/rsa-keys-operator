# KeyGenerator

Making RSA key-pair for annotated deployments and mount them to container.

## TL;DR;

```console
$ git clone https://github.com/akirill0v/rsa-keys-operator.git
helm install key-generator charts/key-generator/ --namespace=kube-system
```
## Uninstalling the Chart

To uninstall/delete the `key-generator`:

```console
$ helm delete key-generator -n kube-system
```
The command removes all the Kubernetes components associated with the chart and deletes the release.

## Configuration

The following table lists the configurable parameters of the KeyGenerator chart and their default values.

| Parameter                  | Description                                                                                                                   | Default                   |
|                            |                                                                                                                               |                           |
| ---------                  | -----------                                                                                                                   | -------                   |
| `image.repository`         | KeyGenerator operator container image                                                                                         | `akirill0v/key_generator` |
| `image.tag`                | KeyGenerator operator container image tag                                                                                     | `latest`                  |
| `replicaCount`             | Number of KeyGenerator operator replicas to create (only 1 is supported)                                                      | `1`                       |
| `imagePullSecrets`         | Specify image pull secrets                                                                                                    | `[]`                      |
| `imagePullPolicy`          | Image pull policy                                                                                                             | `IfNotPresent`            |
| `annotations`              | Annotations applied to operator pod(s)                                                                                        | `[]`                      |
| `nodeSelector`             | Node labels for pod assignment                                                                                                | `{}`                      |
| `tolerations`              | Tolerations used pod assignment                                                                                               | `{}`                      |
| `affinity`                 | Affinity rules for pod assignment                                                                                             | `{}`                      |
| `serviceAccount.create`    | If `true`, create a new service account                                                                                       | `true`                    |
| `serviceAccount.name`      | Service account to be used. If not set and `serviceAccount.create` is `true`, a name is generated using the fullname template | ``                        |
| `resources.requests.cpu`   | CPU resources request                                                                                                         | `100m`                    |
| `resources.request.memory` | Memory resources request                                                                                                      | `100Mi`                   |
| `controller.port`          | Operator controller port to listen                                                                                            | `8080`                    |
| `controller.configPath`    | Where is operator config file                                                                                                 | `/etc/k8s_config/`        |
| `controller.configName`    | Config file name                                                                                                              | `default.yaml`            |
| `config`                   | `controller.configName` file content (in yaml)                                                                                |                           |

Specify each parameter using the `--set key=value[,key=value]` argument to `helm install`. For example:

```console
$ helm install key-generator charts/key-generator/ -n kube-system --set image.tag=vX.X.X
```

Alternatively, a YAML file that specifies the values for the parameters can be provided while
installing the chart. For example:

```console
$ helm install key-generator charts/key-generator/ -n kube-system --values values.yaml
```
