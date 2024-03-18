### Kubernetes with EndpointSlice

Example topology

```
 ┌──────┐                                   Kubernetes
 │ UPS1 ├──┐                                 Cluster       ┌─┐
 └──────┘  │                               ┌─────────┐     │C│
           │     ┌─────────────┐           │ NODE A  │◄───►│L│
 ┌──────┐  │     │ NUT Service │    TCP    ├─────────┤     │I│
 │ UPS2 ├──┼────►│             │◄─────────►│ NODE B  │◄───►│E│
 └──────┘  │     └─────────────┘           ├─────────┤     │N│
           │    192.168.1.12:3493          │ NODE C  │◄───►│T│
 ┌──────┐  │                               └─────────┘     │S│
 │ UPS3 ├──┘                                               └─┘
 └──────┘
```

**endpoint_slice.yaml**

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
    kubernetes.io/service-name: nut-service
addressType: IPv4
ports:
  - name: "nut"
    protocol: TCP
    port: 3493
endpoints:
  - addresses:
      - "192.168.1.12"
```

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
              value: nut-service
            - name: UPSD_PORT
              value: "3493"
            - name: POLL_FREQ
              value: "20"
          resources:
            limits:
              memory: "128Mi"
            requests:
              memory: "128Mi"
          ports:
            - containerPort: 9000
          livenessProbe:
            httpGet:
              path: /probes/health
              port: 9000
            initialDelaySeconds: 5
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /probes/health
              port: 9000

```

