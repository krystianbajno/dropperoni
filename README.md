# DROPPA
When you need to drop a file, it needs to be simple, and **quick**.

Portable cross-platform file upload/download server and an HTTPS reverse proxy written in Rust.

### Features
- Serves files + Web GUI
- Uploads files + Web GUI
- Configurable listening address and port.
- Generates TLS self-signed PKCS8 RSA SHA256 certificates during runtime.
- Can import your custom PEM Private Key and Cert for TLS.
- Runs HTTPS reverse proxy.
- Gets traffic, decrypts traffic, modifies traffic, encrypts traffic, sends traffic.

### Command-Line Arguments

- `--listen <address>` (alias: `--host`) (optional): Specify the IP address to listen on. Default is 0.0.0.0.
- `--port <port>` (optional): Specify the port to listen on. Default is 8000.
- `--directory <dir>` (optional): specify directory to serve. Default is `.`.
- `--tls` (alias: `--ssl`) (optional): generates self-hosted cert in runtime and configures TLS. If specified, the web server will run on `127.0.0.1:<port>`, and the TLS proxy will run on `<listen>:<port>`.
- `--issuer` (optional): set an issuer for self-hosted certificate. Default is getrekt.com
- `--proxy http(s)://<target_address>:<port>` (optional): setup as a reverse proxy.
- `--priv <key>` (optional): setup TLS using custom private key and cert
- `--cert <cert>` (optional): setup TLS using custom private key and cert

### Examples

Share files in current directory
```bash
./droppa
```

**Share files in current directory but encrypted**
```
./droppa --ssl
```

**Setup HTTPS reverse proxy**
```
./droppa --proxy https://hackhack.com:443
```

**More**
```
./droppa # will listen on 0.0.0.0, port 8000, serve current directory, unencrypted
./droppa --directory /usr/share/wordlists # serve directory /usr/share/wordlists
./droppa --listen 192.168.1.10 --port 9999 --tls # will generate custom cert, serve current directory, listen on addr 192.168.1.10, port 9999
./droppa --listen 192.168.1.10 --tls --issuer example.com # will generate custom cert with spoofed example.com issuer
./droppa --listen 192.168.1.10 --cert cert.pem --key key.pem # will use custom private key and cert
./droppa --listen 192.168.1.10 --issuer example.com --proxy https://exampledomain.com:31337 # will serve as reverse proxy, cert generated dynamically, custom issuer
./droppa --listen 192.168.1.10 --proxy https://exampledomain.com:31337 # will serve as reverse proxy, cert generated dynamically
./droppa --listen 192.168.1.10 --cert cert.pem --key key.pem --proxy https://exampledomain.com:31337 # will serve as reverse proxy, will use custom private key and cert
```

### Endpoints

- **`GET /`** - Index files
- **`GET /<file>`** - Download file
- **`POST /`** - Upload file - `enctype="multipart/form-data"`

### MITM
DROPPA is able to perform Man in the Middle. It can get a request from client, decrypt it, process, re-encrypt. and pass it to target.
Modify file mitm_payload.rs. By default it rewrites request Host header to match the target domain. This TLS proxy setup needs that in order to work properly.

It intercepts, decrypts, and logs every request and every response (both headers and body) on proxy and droppa server.

### Bring Your Own Keys
Apart from runtime certificate generation, DROPPA has capability to load your own certificates.

OpenSSL oneliner bonus
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 3650 -nodes -subj "/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=CommonNameOrHostname" -nodes
```
