# Kubernetes: Connecting with an EndpointSlice

If your NUT (Network UPS Tools) service is running outside the Kubernetes cluster, you can use a `Service` and `EndpointSlice` to make it discoverable within the cluster. This approach allows you to refer to the external service by a local Kubernetes DNS name (e.g., `nut-service`), just as you would with any other service running inside the cluster.

## Topology

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

## 1. Exposing the External NUT Service

First, create a `Service` without a selector and an `EndpointSlice` that points to the external IP address of your NUT service. This makes the external service available at `nut-service.default.svc.cluster.local` (or simply `nut-service` from within the same namespace).

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

Now, you can deploy `nut_webgui` and configure it to connect to the NUT service using the Kubernetes service name (`nut-service`) you just created.

The following manifests define the `nut_webgui` Deployment, its Service, and the Secret for its credentials.

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

## Applying the Configuration

You can apply these manifest files to your cluster using `kubectl`.

```bash
kubectl apply -f nut-external-service.yaml
kubectl apply -f nut-webgui-secret.yaml
kubectl apply -f nut-webgui-service.yaml
kubectl apply -f nut-webgui-deployment.yaml
```
