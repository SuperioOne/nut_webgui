# Using NUT with TLS

**`docker-compose.yaml` Example:**
```yaml
version: "3.3"
services:
  nutweb:
    image: ghcr.io/superioone/nut_webgui:latest
    restart: always
    ports:
      - 9000:9000
    environment:
      UPSD_ADDR: "your-nut-server"             # Address of the NUT server
      UPSD_USER: "username"
      UPSD_PASS: "password"
      UPSD_TLS: "strict"                       # Enable strict TLS mode
      UPSD_ROOT_CA: "/run/secrets/root_ca"     # Path to the Root CA certificate
    secrets:
      - upsd_username
      - upsd_password
      - config_file
      - root_ca

secrets:
  root_ca:
    file: ./rootca.crt                         # The exported Root CA from the NUT server
```

**`docker/podman run` Example:**
```shell
chmod o=+r rootca.crt;

docker run \
  -e UPSD_ADDR=your-nut-server \
  -e UPSD_PASS=password \
  -e UPSD_USER=username \
  -e UPSD_TLS=strict \
  -e UPSD_ROOT_CA=/cert/rootca.crt \
  -v "$(pwd)/rootca.crt":/cert/rootca.crt \
  -p 9000:9000 \
  ghcr.io/superioone/nut_webgui:latest
```

## Adding Root CA

When connecting to a NUT server that uses a certificate, the `nut_webgui` container needs to trust the Root Certificate Authority (CA) that signed the server's certificate.

You can provide the Root CA by mounting the certificate file into the container and setting the `UPSD_ROOT_CA` environment variable to its path.

> **Important:** The `nut_webgui` container runs as a non-root user. Ensure the mounted certificate file has appropriate read permissions (e.g., `chmod 644 rootca.crt`).

The container image automatically symlinks the certificate provided via `UPSD_ROOT_CA` to the system's trust store location at `/usr/local/ssl/cert.pem`.

## TLS Verification Modes

The `UPSD_TLS` environment variable controls the TLS verification behavior:

*   `disable`: Disables TLS and uses an insecure TCP connection.
*   `skip`: Enables TLS but skips certificate validation. This is insecure and should only be used for testing.
*   `strict`: Enables TLS with full certificate validation. This is the recommended mode for production. It requires providing a Root CA via the `UPSD_ROOT_CA` variable.

# Bonus: Configuring Self-Signed TLS on the NUT Server

This section details how to generate self-signed certificates for your NUT server.

## Steps to Generate Certificates (NSS Backend)

The following example uses the `certutil` tool for the NSS backend, and it won't work if your `nut` package is built with `--with-openssl`. 
These commands should be run on the NUT server machine.

1.  **Create a Certificate Database**

    First, create a directory for your database and initialize it. You will be prompted to set a password for the database.

    ```bash
    mkdir cert_db
    certutil -N -d cert_db
    ```

2.  **Create a Self-Signed Root CA**

    Next, generate the Root CA certificate.

    ```bash
    certutil -S -d cert_db -n "My Root CA" -s "CN=My CA,O=MyCompany,ST=MyState,C=US" -t "CT,," -x -2 -v 24
    ```
    *   `-t "CT,,"`: Specifies this certificate trust attributes (CT -> Trusted CA and trusted CA for client authentication).
    *   `-x`: Creates a self-signed certificate.
    *   `-2`: Adds X.509 certificate constraint extension.
    *   `-v`: Number of months root certificate is valid for.

    > **Note:** `certutil` may ask, "Is this a CA certificate? [y/N]". You **must** answer `y` to create a valid Root CA.

3.  **Export the Root CA Certificate**

    Export the public part of the Root CA so it can be shared with clients (like `nut_webgui`).

    ```bash
    certutil -L -d cert_db -n "My Root CA" -a -o rootca.crt
    ```
    This `rootca.crt` file is what you will use for the `UPSD_ROOT_CA` variable in `nut_webgui`.

4.  **Create a Server Certificate Signing Request (CSR)**

    Now, create a request for the server certificate.

    ```bash
    certutil -R \
      -k rsa \
      -g 2048 \
      -s "CN=localhost,O=MyCompany,ST=MyState,C=US" \
      --extSAN dns:my-server,dns:localhost,ip:192.168.221.42,ip:127.0.0.1 \
      -d cert_db \
      -a \
      -o server.req
    ```
    > **Critical:** The Common Name (`CN`) and Subject Alternative Names (`--extSAN`) must include the hostname(s) and IP address(es) that clients will use to 
    connect to the NUT server. This value must match what is configured in `nut_webgui`'s `UPSD_ADDR` environment variable or the `[upsd].address` field in `config.toml`.

5.  **Issue the Server Certificate**

    Sign the CSR with your Root CA to create the server certificate.

    ```bash
    certutil -C -d cert_db -c "My Root CA" -a -i server.req -o server.crt -2 -6 "serverAuth" -v 24 
    ```
    *   `-c "My Root CA"`: Specifies the issuer certificate.
    *   `-6`: Sets the Extended Key Usage (EKU) for Server Authentication.
    *   `-v`: Number of months server certificate is valid for.

6.  **Add the Server Certificate to the Database**

    Import the newly created server certificate into the database.

    ```bash
    certutil -A -d cert_db -n "nut server" -a -i server.crt -t ',,'
    ```

7.  **Deploy the Certificate Database**

    Copy the database to the NUT configuration directory and set the correct ownership and permissions.

    ```bash
    install -d /usr/local/ups/etc
    cp -r cert_db /usr/local/ups/etc/
    chown -R root:nut /usr/local/ups/etc/cert_db
    chmod -R 774 /usr/local/ups/etc/cert_db
    ```

8.  **Configure `upsd.conf`**

    Finally, edit `/etc/nut/upsd.conf` to tell NUT how to use the certificates.

    ```ini
    # Path to the certificate database
    CERTPATH /usr/local/ups/etc/cert_db

    # Nickname of the server certificate and the database password
    CERTIDENT "nut server" "database_password"

    # (Optional but recommended) Disable weak SSL/TLS protocols
    DISABLE_WEAK_SSL true
    ```
    *   Replace `"nut server"` with the nickname used in step 6.
    *   Replace `"database_password"` with the password you created in step 1.


Reference: [Network Ups Tools - User Manual](https://networkupstools.org/docs/user-manual.chunked/NUT_Security.html#_configuring_ssl)
