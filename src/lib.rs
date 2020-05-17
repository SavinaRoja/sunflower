use std::sync::Mutex;
use std::collections::HashMap;
use std::net::Ipv4Addr;

#[macro_use]
extern crate serde_derive;
extern crate serde_bencode;

// Resources
// https://www.bittorrent.org/beps/bep_0003.html
// https://wiki.theory.org/index.php/BitTorrentSpecification#Tracker_HTTP.2FHTTPS_Protocol


pub struct AppState {
    pub min_interval: u16,
    pub peermap: Mutex<HashMap<String,Vec<Peer>>>, // <- Mutex is necessary to mutate safely across threads
}

// TODO: Think of a better strategy for locking and data structure


//An example of this GET message could be:
//http://some.tracker.com:999/announce
// ?info_hash=12345678901234567890
// &peer_id=ABCDEFGHIJKLMNOPQRST
// &ip=255.255.255.255
// &port=6881
// &downloaded=1234
// &left=98765
// &event=Stopped

// Things that I think are a bare minimum: info_hash, peer_id, port, downloaded, left


#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Event{
    Started,
    Completed,
    Stopped
}

#[derive(Deserialize, Debug)]
pub struct AnnounceParams {
    pub info_hash: String,
    pub peer_id: String,
    pub port: u16,
    pub downloaded: u64,  // By convention, bytes downloaded
    pub uploaded: Option<u64>,  // By convention, bytes uploaded
    pub left: u64,  // Bytes remaining to download before complete
    pub compact: Option<String>,
    pub no_peer_id: Option<String>,
    pub event: Option<Event>,
    pub ip: Option<String>,
    pub numwant: Option<u16>,
    pub key: Option<String>,
    pub trackerid: Option<String>,
}

// TODO: Figure out how to cast the "0" or "1" to a bool while maintaining optionality


#[derive(Serialize, Deserialize, Debug)]
pub struct AnnounceResponse {
    #[serde(rename = "failure reason")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    #[serde(rename = "warning message")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers: Option<Vec<Peer>>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peer{
    #[serde(rename = "peer id")]
    pub peer_id: String,
    pub ip: Ipv4Addr,
    // pub ip: String,
    pub port: u16,
}


impl Default for AnnounceResponse {
    fn default() -> AnnounceResponse {
        AnnounceResponse {
            failure_reason: None,
            warning_message: None,
            interval: None,
            peers: None,
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
