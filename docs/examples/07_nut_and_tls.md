# Using NUT with TLS

This guide explains how to configure `nut_webgui` and the NUT server for secure communication using TLS.

## `nut_webgui` Configuration

The primary way to configure `nut_webgui` is through environment variables, which can be easily managed using Docker Compose.

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
      CONFIG_FILE: "/run/secrets/config_file"
      UPSD_ADDR: "your-nut-server"             # Address of the NUT server
      UPSD_USER: "/run/secrets/upsd_username"
      UPSD_PASS: "/run/secrets/upsd_password"
      UPSD_TLS: "strict"                       # Enable strict TLS mode
      UPSD_ROOT_CA: "/run/secrets/root_ca"     # Path to the Root CA certificate
    secrets:
      - upsd_username
      - upsd_password
      - config_file
      - root_ca

secrets:
  upsd_username:
    file: ./upsd_user.txt
  upsd_password:
    file: ./upsd_password.txt
  config_file:
    file: ./config.toml
  root_ca:
    file: ./rootca.crt                         # The exported Root CA from the NUT server
```

### Using a Custom Root CA

When connecting to a NUT server that uses a certificate, the `nut_webgui` container needs to trust the Root Certificate Authority (CA) that signed the server's certificate.

You can provide the Root CA by mounting the certificate file into the container and setting the `UPSD_ROOT_CA` environment variable to its path.

> **Important:** The `nut_webgui` container runs as a non-root user. Ensure the mounted certificate file has appropriate read permissions (e.g., `chmod 644 rootca.crt`).

The container image automatically symlinks the certificate provided via `UPSD_ROOT_CA` to the system's trust store location at `/usr/local/ssl/cert.pem`.

### TLS Verification Modes

The `UPSD_TLS` environment variable controls the TLS verification behavior:

*   `disabled`: Disables TLS and uses an insecure TCP connection.
*   `skip`: Enables TLS but skips certificate validation. This is insecure and should only be used for testing.
*   `strict`: Enables TLS with full certificate validation. This is the recommended mode for production. It requires providing a Root CA via the `UPSD_ROOT_CA` variable.


## Bonus: Configuring Self-Signed TLS on the NUT Server

This section details how to generate self-signed certificates for your NUT server.

The NUT project supports two TLS backends: `--with-openssl` and `--with-nss`. The following example uses the `certutil` tool from the NSS backend. These commands should be run on the NUT server machine.

### Steps to Generate Certificates

1.  **Create a Certificate Database**

    First, create a directory for your database and initialize it. You will be prompted to set a password for the database.

    ```bash
    mkdir cert_db
    certutil -N -d cert_db
    ```

2.  **Create a Self-Signed Root CA**

    Next, generate the Root CA certificate.

    ```bash
    certutil -S -d cert_db -n "My Root CA" -s "CN=My CA,O=MyCompany,ST=MyState,C=US" -t "CT,," -x -2
    ```
    *   `-t "CT,,"`: Specifies this certificate is a trusted CA for signing.
    *   `-x`: Creates a self-signed certificate.
    *   `-2`: Sets a 2-year validity period.

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
    certutil -R -s "CN=localhost,O=MyCompany,ST=MyState,C=US" --extSAN dns:my-server,dns:localhost,ip:192.168.221.42,ip:127.0.0.1 -d cert_db -a -o server.req
    ```
    > **Critical:** The Common Name (`CN`) and Subject Alternative Names (`--extSAN`) must include the hostname(s) and IP address(es) that clients will use to 
    connect to the NUT server. This value must match what is configured in `nut_webgui`'s `UPSD_ADDR` environment variable or the `[upsd].address` field in `config.toml`.

5.  **Issue the Server Certificate**

    Sign the CSR with your Root CA to create the server certificate.

    ```bash
    certutil -C -d cert_db -c "My Root CA" -a -i server.req -o server.crt -2 -6
    ```
    *   `-c "My Root CA"`: Specifies the issuer certificate.
    *   `-6`: Sets the Extended Key Usage (EKU) for Server Authentication.

6.  **Add the Server Certificate to the Database**

    Import the newly created server certificate into the database.

    ```bash
    certutil -A -d cert_db -n "My nut server" -a -i server.crt -t ',,'
    ```

7.  **Deploy the Certificate Database**

    Copy the database to the NUT configuration directory and set the correct ownership and permissions.

    ```bash
    sudo cp -r cert_db /usr/local/ups/etc/
    sudo chown -R root:nut /usr/local/ups/etc/cert_db
    sudo chmod -R 774 /usr/local/ups/etc/cert_db
    ```

8.  **Configure `upsd.conf`**

    Finally, edit `upsd.conf` to tell NUT how to use the certificates.

    ```ini
    # Path to the certificate database
    CERTPATH /usr/local/ups/etc/cert_db

    # Nickname of the server certificate and the database password
    CERTIDENT "My nut server" "database_password"

    # (Optional but recommended) Disable weak SSL/TLS protocols
    DISABLE_WEAK_SSL true
    ```
    *   Replace `"My nut server"` with the nickname used in step 6.
    *   Replace `"database_password"` with the password you created in step 1.

Reference: [Network Ups Tools - User Manual](https://networkupstools.org/docs/user-manual.chunked/NUT_Security.html#_configuring_ssl)
