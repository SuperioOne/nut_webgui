# Kubernetes: Connecting with an EndpointSlice

When your NUT service runs outside the cluster, you can use a `Service` and
`EndpointSlice` to expose it internally. This enables DNS-based discovery within
the cluster (e.g., `nut-service`).


Example topology:
```
 ┌──────┐                                   Kubernetes
 │ UPS1 ├──┐                                 Cluster
 └──────┘  │                               ┌─────────┐
           │     ┌─────────────┐           │ NODE A  │◄───► ┌─────────┐
 ┌──────┐  │     │ NUT Service │    TCP    ├─────────┤      │         │
 │ UPS2 ├──┼────►│             │◄─────────►│ NODE B  │◄───► │ INGRESS │
 └──────┘  │     └─────────────┘           ├─────────┤      │         │
           │    192.168.1.12:3493          │ NODE C  │◄───► └─────────┘
 ┌──────┐  │                               └─────────┘
 │ UPS3 ├──┘
 └──────┘
```

## 1. Exposing the external NUT service

Create a headless `Service` and an `EndpointSlice` pointing to your external NUT
daemon. This resolves `nut-service` (`nut-service.default.svc.cluster.local`) to
the external IP inside the cluster.

**nut-external-service.yaml**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: nut-service
spec:
  ports:
    - name: nut
      protocol: TCP
      port: 3493
      targetPort: 3493
---
apiVersion: discovery.k8s.io/v1
kind: EndpointSlice
metadata:
  name: nut-service-1
  labels:
    # This label connects the EndpointSlice to the Service.
    kubernetes.io/service-name: nut-service
addressType: IPv4
ports:
  - name: "nut"
    protocol: TCP
    port: 3493
endpoints:
  - addresses:
      # The external IP address of your NUT service.
      - "192.168.1.12"
  # You can configure other topology-aware routing features here.
```

## 2. Deploying nut_webgui

Configure `nut_webgui` to route traffic to the internal `nut-service` DNS name.

**nut-webgui-secret.yaml**
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

**nut-webgui-service.yaml**
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

**nut-webgui-deployment.yaml**
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
              # Use the Kubernetes service name to connect to the NUT service.
              value: nut-service
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

## 3. Applying the manifest files

```bash
kubectl apply -f nut-external-service.yaml
kubectl apply -f nut-webgui-secret.yaml
kubectl apply -f nut-webgui-service.yaml
kubectl apply -f nut-webgui-deployment.yaml
```
