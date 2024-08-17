# Droppa
Rust file upload/download server and reverse HTTPS proxy. 

### Features
- Serves files
- Uploads files
- Configurable listening address and port.
- Generates TLS self-signed PKCS8 RSA SHA256 certificates during runtime.
- Creates encrypted TLS reverse proxy.

### Command-Line Arguments
```bash
--listen (optional): Specify the IP address to listen on. Default is 0.0.0.0.
--port (optional): Specify the port to listen on. Default is 8080.
--directory (optional): specify directory to serve.
--tls (optional): configures TLS. If specified, the web server will run on 127.0.0.1:<port>, and the TLS proxy will run on <listen>:<port>.
--issuer (optional): set an issuer for self-hosted certificate. Default is getrekt.com

./droppa --listen 0.0.0.0 --port 8000 --tls --directory .
```
