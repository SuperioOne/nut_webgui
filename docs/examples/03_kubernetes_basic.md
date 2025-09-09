# Kubernetes: Basic Deployment

## Topology

The diagram below illustrates a common scenario where the NUT (Network UPS Tools) service is accessible over the network.

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

This example consists of three main components: a Secret, a Service, and a Deployment.

> **Note:** This is a minimal setup and does not include an [Ingress configuration](https://kubernetes.io/docs/concepts/services-networking/ingress/) for exposing the service outside the cluster. You will need to configure that separately based on your environment.

---

## 1. Secret

First, create a Secret to store sensitive data, such as the username and password for the NUT service. Replace the placeholder values with your actual credentials, encoded in Base64.

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

Next, define a Service to expose the `nut_webgui` deployment within the cluster. This allows other services to communicate with it.

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

Finally, create the Deployment, which defines the desired state for your application. This includes the container image, environment variables, resource limits, and health probes.

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

## Applying the Configuration

You can apply these manifest files to your cluster:

```bash
kubectl apply -f secret.yaml
kubectl apply -f service.yaml
kubectl apply -f deployment.yaml
```

