# Kubernetes - Basic

Example topology

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

*This example only provides basic setup. It doesn't provide any [ingress configuration.](https://kubernetes.io/docs/concepts/services-networking/ingress/)*

**secret.yaml**

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: nutweb-secret
type: Opaque
data:
  UPSD_USER: Zm9v  # base64 encoded "foo"
  UPSD_PASS: YmFy  # base64 encoded "bar"
```

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

