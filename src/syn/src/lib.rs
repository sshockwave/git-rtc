use ::webrtc::{
    api::{setting_engine, APIBuilder},
    ice_transport::ice_server::RTCIceServer,
    peer_connection::configuration::RTCConfiguration,
    error::Result,
};

pub enum ObjectID {
    SHA1([u32; 5]),
}

struct Peer {
    api: ::webrtc::api::API,
}

struct Endpoint {
    peers: Vec<Peer>,
}

impl Peer {
    async fn new() -> Result<()> {
        let setting_engine = setting_engine::SettingEngine::default();
        let api = APIBuilder::new()
            .with_setting_engine(setting_engine)
            .build();
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };
        let peer_connection = api.new_peer_connection(config).await?;
        Ok(())
    }
    async fn new_from_sdp() {
        use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
    }
}

impl Endpoint {
    async fn new() {}
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
