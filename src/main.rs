use sunflower::{AnnounceParams, AnnounceResponse, AppState, Peer};

use actix_web::{middleware, web, App, HttpRequest, HttpServer};
use actix_web::web::{Query};
use std::collections::{HashMap};
use std::sync::Mutex;
use std::net::Ipv4Addr;
use std::str::FromStr;


async fn index(data: web::Data<AppState>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Request number: {}", counter) // <- response with count
}

async fn announce(req: HttpRequest, data: web::Data<AppState>, info: Query<AnnounceParams>) -> String {

    let mut response = AnnounceResponse::default();

    let mut peermap = data.peermap.lock().unwrap();

    let ip_address = Ipv4Addr::from_str(req.connection_info().remote().unwrap().split(':').next().unwrap()).unwrap();

    response.interval = Some(data.min_interval);

    match peermap.get_mut(info.info_hash.as_str()) {
        Some(peers) => {
            let mut ip_found = false;
            let mut response_peers: Vec<Peer> = Vec::new();
            for peer in peers.into_iter() {
                if peer.ip == ip_address {
                    ip_found = true;
                } else {
                    response_peers.push(peer.clone());
                }
            }
            if !ip_found {
                peers.push(Peer {
                        peer_id: info.peer_id.clone(),
                        ip: ip_address,
                        port: info.port,
                    }
                );
            }
            response.peers = Some(response_peers);
            let serialized = serde_bencode::to_string(&response).unwrap();
            println!("{}", serialized);
            return serialized
        },
        None => {
            peermap.insert(
                info.info_hash.clone(),
                vec![Peer{
                    peer_id: info.peer_id.clone(),
                    ip: ip_address,
                    port: info.port,
                }]
            );
            response.peers = Some(vec![]);
            let serialized = serde_bencode::to_string(&response).unwrap();
            println!("{}", serialized);
            return serialized
        }
    }
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();


    let counter = web::Data::new(AppState {
        counter: Mutex::new(0),
        min_interval: 30,
        peermap: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(counter.clone())
            .service(web::resource("/").to(index))
            .service(web::resource("/announce").to(announce))
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::{http, test, web, App, Error};

    #[actix_rt::test]
    async fn test_index() -> Result<(), Error> {
        let app = App::new().route("/", web::get().to(index));
        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r##"Hello world!"##);

        Ok(())
    }
}
