# Droppa
Cross-platform file upload/download server and an HTTPS reverse proxy written in Rust.

### Features
- Serves files
- Uploads files
- Configurable listening address and port.
- Generates TLS self-signed PKCS8 RSA SHA256 certificates during runtime.
- Runs HTTPS reverse proxy.
- Gets traffic, decrypts traffic, (MITM), encrypts traffic, sends traffic.

### Command-Line Arguments

- `--listen <address>` (optional): Specify the IP address to listen on. Default is 0.0.0.0.
- `--port <port>` (optional): Specify the port to listen on. Default is 8000.
- `--directory <dir>` (optional): specify directory to serve. Default is `.`.
- `--tls` (alias: `--ssl`) (optional): configures TLS. If specified, the web server will run on `127.0.0.1:<port>`, and the TLS proxy will run on `<listen>:<port>`.
- `--issuer` (optional): set an issuer for self-hosted certificate. Default is getrekt.com
- `--proxy http(s)://<target_address>:<port>` (optional): setup as a reverse proxy.
```bash
./droppa --listen 0.0.0.0 --port 8000 --tls --issuer example.com --proxy https://31.3.3.7:31337 --directory .
```

### Endpoints


**`GET /`** - Index files
**`GET /<file>`** - Download file
**`POST /`** - Upload file - `enctype="multipart/form-data"`
