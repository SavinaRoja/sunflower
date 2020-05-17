use sunflower::{AnnounceParams, AnnounceResponse, AppState, Peer};

use actix_web::{middleware, web, App, HttpRequest, HttpServer};
use actix_web::web::{Query};
use std::collections::{HashMap};
use std::sync::Mutex;
use std::net::Ipv4Addr;
use std::str::FromStr;


async fn announce(req: HttpRequest, data: web::Data<AppState>, info: Query<AnnounceParams>) -> String {

    let mut response = AnnounceResponse::default();

    let mut peermap = data.peermap.lock().unwrap();

    println!("{:?}", req.connection_info().remote());
    println!("{:?}", req.connection_info().remote().unwrap());

    let ip_address = Ipv4Addr::from_str(
        req.connection_info().remote().unwrap().split(':').next().unwrap()).unwrap();
    // let ip_address = String::from(req.connection_info().remote().unwrap());

    response.interval = Some(data.min_interval);

    match peermap.get_mut(info.info_hash.as_str()) {
        Some(peers) => {
            let mut ip_found = false;
            let mut response_peers: Vec<Peer> = Vec::new();
            for peer in peers.iter() {
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


    let app_data = web::Data::new(AppState {
        min_interval: 30,
        peermap: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(app_data.clone())
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
    async fn test_announce() -> Result<(), Error> {
        let app_data = web::Data::new(AppState {
            min_interval: 30,
            peermap: Mutex::new(HashMap::new()),
        });
        let app = App::new()
            .app_data(app_data.clone())
            .route("/announce", web::get().to(announce));
        let mut app = test::init_service(app).await;

        // info_hash, peer_id, port, downloaded, left
        let req = test::TestRequest::get().uri("/announce?info_hash=bobloblawlawblog&peer_id=somepeerid&downloaded=0&left=1024&port=9023")
            .header("Forwarded", "88.88.88.88")
            .header("X-Forwarded-For", "88.88.88.88")
            .to_request();
        println!("{:?}", req);
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r##"d8:intervali30e5:peerslee"##);

        // info_hash, peer_id, port, downloaded, left
        let req = test::TestRequest::get().uri("/announce?info_hash=bobloblawlawblog&peer_id=somepeerid&downloaded=0&left=1024&port=9023")
            .header("Forwarded", "99.99.99.99")
            .header("X-Forwarded-For", "99.99.99.99")
            .to_request();
        println!("{:?}", req);
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r##"d8:intervali30e5:peersld2:ip11:88.88.88.887:peer id10:somepeerid4:porti9023eeee"##);

        Ok(())
    }
}
