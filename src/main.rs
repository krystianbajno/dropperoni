use std::{path::PathBuf, sync::Arc};
use clap::{Arg, Command};

mod server;
mod proxy;
mod views;
mod controller;
mod routes;
mod ssl; 

#[tokio::main]
async fn main() {
    let matches = Command::new("DROPPA")
        .version("1.0")
        .author("Krystian Bajno")
        .about("A simple file server server with optional TLS")
        .arg(Arg::new("listen")
            .long("listen")
            .value_name("ADDRESS")
            .help("Set the listening address")
            .default_value("0.0.0.0")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("port")
            .long("port")
            .value_name("PORT")
            .help("Set the listening port")
            .default_value("8000")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("directory")
            .long("directory")
            .short('d')
            .value_name("DIRECTORY")
            .help("Set the directory to serve files from")
            .default_value(".")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("tls")
            .long("tls")
            .alias("ssl")
            .help("Enable TLS")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("issuer")
            .long("TLS_ISSUER")
            .help("Set TLS issuer")
            .default_value("getrekt.com")
            .action(clap::ArgAction::Set))
        .get_matches();

    let listen_address = matches.get_one::<String>("listen").unwrap();
    let port = matches.get_one::<String>("port").unwrap();
    let directory = matches.get_one::<String>("directory").unwrap();
    let enable_ssl = matches.get_one::<bool>("tls").unwrap();
    let issuer = matches.get_one::<String>("issuer").unwrap();
    println!("{}", enable_ssl);

    let dir = Arc::new(PathBuf::from(directory));

    if *enable_ssl {
        let target_address = format!("127.0.0.1:{}", port);
        server::start_rouille_server(target_address.clone(), dir.clone());

        let proxy_address = format!("{}:{}", listen_address, port);
        println!("DROPPA: TLS Proxy running on https://{} from directory {}", proxy_address, directory);

        match proxy::start_ssl_proxy(&proxy_address, &target_address, &issuer).await {
            Ok(()) => println!("OK"),
            Err(err) => println!("{:?}", err),
        };
    } else {
        let server_address = format!("{}:{}", listen_address, port);
        server::start_rouille_server(server_address.clone(), dir.clone());
        println!("DROPPA: Serving on http://{} from directory {}", server_address, directory);
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
