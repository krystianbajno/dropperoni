use std::{path::PathBuf, sync::Arc};
use rouille;

use crate::routes;

pub fn start_rouille_server(address: String, dir: Arc<PathBuf>) {
    std::thread::spawn(move || {
        rouille::start_server(&address, move |request| {
            routes::handle_request(request, &dir)
        });
    });
}
