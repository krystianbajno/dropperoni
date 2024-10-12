mod views;
mod transport;
mod proxy;
mod mitm;
mod http;
mod crypto;

use std::{path::PathBuf, sync::Arc};
use clap::{Arg, Command};
use http::server;
use proxy::proxy::start_ssl_proxy;

// use simplelog::*;
// use std::fs::File;

#[tokio::main]
async fn main() {
    // let log_file = File::create("droppa.log").unwrap();
    // CombinedLogger::init(vec![
        // WriteLogger::new(LevelFilter::Info, Config::default(), log_file),
    // ]).unwrap();

    let matches = Command::new("DROPPA")
        .version("1.0")
        .author("Krystian Bajno")
        .about("A simple file server server with optional TLS")
        .arg(Arg::new("listen")
            .long("listen")
            .alias("host")
            .value_name("listen")
            .help("Set the listening address")
            .default_value("0.0.0.0")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("port")
            .long("port")
            .value_name("port")
            .help("Set the listening port")
            .default_value("8000")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("directory")
            .long("directory")
            .short('d')
            .value_name("directory")
            .help("Set the directory to serve files from")
            .default_value(".")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("tls")
            .long("tls")
            .alias("ssl")
            .help("Enable TLS")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("issuer")
            .long("issuer")
            .help("Set TLS issuer")
            .default_value("getrekt.com")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("proxy")
            .long("proxy")
            .help("Setup as reverse proxy")
            .value_name("proxy")
            .default_value("")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("priv")
            .long("priv")
            .value_name("private_key")
            .help("Path to the private key file")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("cert")
            .long("cert")
            .value_name("certificate")
            .help("Path to the certificate file")
            .action(clap::ArgAction::Set))
        .get_matches();

    let listen_address = matches.get_one::<String>("listen").unwrap();
    let port = matches.get_one::<String>("port").unwrap();
    let directory = matches.get_one::<String>("directory").unwrap();
    let enable_ssl = matches.get_one::<bool>("tls").unwrap();
    let issuer = matches.get_one::<String>("issuer").unwrap();
    let proxy_target_addr = matches.get_one::<String>("proxy").unwrap();

    let dir = Arc::new(PathBuf::from(directory));

    let private_key_path = matches.get_one::<String>("priv").map(PathBuf::from);
    let cert_path = matches.get_one::<String>("cert").map(PathBuf::from);

    if should_start_tls_proxy(enable_ssl, proxy_target_addr, &private_key_path, &cert_path) {
        start_tls_proxy(listen_address, port, dir.clone(), issuer, private_key_path.clone(), cert_path.clone()).await;
    } 

    if should_start_plain_server(enable_ssl, proxy_target_addr, &private_key_path, &cert_path)  {
        start_plain_server(listen_address, port, dir.clone());
    }

    if !proxy_target_addr.trim().is_empty() {
        start_reverse_proxy(listen_address, port, proxy_target_addr, issuer, private_key_path, cert_path).await;
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn should_start_tls_proxy(enable_ssl: &bool, proxy_target_addr: &str, private_key_path: &Option<PathBuf>, cert_path: &Option<PathBuf>) -> bool {
    proxy_target_addr.is_empty() && (*enable_ssl || (private_key_path.is_some() && cert_path.is_some()))
}

fn should_start_plain_server(enable_ssl: &bool, proxy_target_addr: &str, private_key_path: &Option<PathBuf>, cert_path: &Option<PathBuf>) -> bool {
    proxy_target_addr.is_empty() && (!*enable_ssl || !(private_key_path.is_some() || cert_path.is_some()))
}


async fn start_tls_proxy(
    listen_address: &str,
    port: &str,
    dir: Arc<PathBuf>,
    issuer: &str,
    private_key_path: Option<PathBuf>,
    cert_path: Option<PathBuf>,
) {
    let target_address = format!("127.0.0.1:{}", port);

    server::start_rouille_server(target_address.clone(), dir.clone());

    let proxy_address = format!("{}:{}", listen_address, port);
    println!("DROPPA: TLS Proxy running on https://{}, from directory {}", proxy_address, dir.clone().display());

    match start_ssl_proxy(&proxy_address, &target_address, issuer, private_key_path.as_deref(), cert_path.as_deref()).await {
        Ok(()) => println!("OK"),
        Err(err) => println!("{:?}", err),
    };
}

async fn start_reverse_proxy(
    listen_address: &str,
    port: &str,
    proxy_target_addr: &str,
    issuer: &str,
    private_key_path: Option<PathBuf>,
    cert_path: Option<PathBuf>,
) {
    let target_address = format!("{}", proxy_target_addr);
    let proxy_address = format!("{}:{}", listen_address, port);
    println!("DROPPA: TLS Proxy running on https://{} -> targeting {}", proxy_address, target_address);

    match start_ssl_proxy(&proxy_address, &target_address, issuer, private_key_path.as_deref(), cert_path.as_deref()).await {
        Ok(()) => println!("OK"),
        Err(err) => println!("{:?}", err),
    };
}

fn start_plain_server(listen_address: &str, port: &str, dir: Arc<PathBuf>) {
    let server_address = format!("{}:{}", listen_address, port);
    server::start_rouille_server(server_address.clone(), dir.clone());
    println!("DROPPA: Serving on http://{} from directory {}", server_address, dir.display());
}