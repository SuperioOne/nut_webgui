# Kubernetes: Basic Deployment

Example topology:
```
 ┌──────┐                                   Kubernetes
 │ UPS1 ├──┐                                 Cluster
 └──────┘  │                               ┌─────────┐
           │     ┌─────────────┐           │ NODE A  │◄───► ┌─────────┐
 ┌──────┐  │     │ NUT Service │    TCP    ├─────────┤      │         │
 │ UPS2 ├──┼────►│             │◄─────────►│ NODE B  │◄───► │ INGRESS │
 └──────┘  │     └─────────────┘           ├─────────┤      │         │
           │     192.128.0.1:3493          │ NODE C  │◄───► └─────────┘
 ┌──────┐  │                               └─────────┘
 │ UPS3 ├──┘
 └──────┘
```

> **Note:** This example covers the application deployment. Exposing the service
> outside the cluster requires a separate
> [Ingress configuration](https://kubernetes.io/docs/concepts/services-networking/ingress/).

## 1. Secret

Create a Secret to store sensitive data, such as the username and password for
the NUT service.

**secret.yaml**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: nutweb-secret
type: Opaque
data:
  UPSD_USER: Zm9v  # "foo" in base64
  UPSD_PASS: YmFy  # "bar" in base64
```

## 2. Service

Create a Service to expose the `nut_webgui` deployment within the cluster. This
allows other services to communicate with it (mainly required for ingress).

**service.yaml**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: nutwebgui-service
spec:
  type: ClusterIP
  selector:
    app: nutweb-pod
  ports:
  - port: 80
    targetPort: 9000
```

## 3. Deployment

Finally, create the Deployment to define the application state, including the
image, configuration, volumes, and health probes.

**deployment.yaml**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nutweb-deployment
spec:
  replicas: 1
  selector:
    matchLabels:
      app: nutweb-pod
  template:
    metadata:
      labels:
        app: nutweb-pod
    spec:
      containers:
        - name: nutweb
          image: ghcr.io/superioone/nut_webgui:latest
          envFrom:
            - secretRef:
                name: nutweb-secret
          env:
            - name: UPSD_ADDR
              value: "192.128.0.1"
            - name: UPSD_PORT
              value: "3493"
            - name: POLL_FREQ
              value: "20"
          resources:
            limits:
              memory: "64Mi"
            requests:
              memory: "32Mi"
          ports:
            - containerPort: 9000
          startupProbe:
            httpGet:
              path: /probes/readiness
              port: 9000
            initialDelaySeconds: 5
            failureThreshold: 15
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /probes/health
              port: 9000
            initialDelaySeconds: 5
            failureThreshold: 3
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /probes/readiness
              port: 9000
            initialDelaySeconds: 5
            failureThreshold: 3
            periodSeconds: 30
```

## 4. Applying the manifest files

```bash
kubectl apply -f secret.yaml
kubectl apply -f service.yaml
kubectl apply -f deployment.yaml
```
