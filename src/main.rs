use anyhow::Result;

use libp2p::{
    futures::StreamExt,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::{SwarmBuilder, SwarmEvent},
};
use libp2p_quic::{generate_keypair, QuicConfig, ToLibp2p};

use tracing::{info, Level};
use tracing_subscriber;

type Crypto = libp2p_quic::TlsCrypto;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .init();

    log_panics::init();

    let keypair = generate_keypair();
    let peer_id = keypair.to_peer_id();

    let transport = QuicConfig::<Crypto>::new(keypair)
        .listen_on("/ip4/192.168.25.64/udp/0/quic".parse()?)
        .await?
        .boxed();

    let behaviour = Mdns::new(MdnsConfig::default()).await?;

    let mut swarm = SwarmBuilder::new(transport, behaviour, peer_id)
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    swarm.listen_on("/ip4/192.168.25.64/udp/0/quic".parse()?)?;

    loop {
        if let SwarmEvent::Behaviour(event) = swarm.select_next_some().await {
            match event {
                MdnsEvent::Discovered(peers) => peers
                    .into_iter()
                    .for_each(|(peer, addr)| info!("Обнaружен: heer_id: {} addr: {}", peer, addr)),
                MdnsEvent::Expired(peers) => peers
                    .into_iter()
                    .for_each(|(peer, addr)| info!("Потерян: heer_id: {} addr: {}", peer, addr)),
            }
        }
    }
}
