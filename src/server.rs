use base64;
use futures::{future, prelude::*};
use identity::ed25519;
use identity::Keypair;
use libp2p::{mplex, plaintext, identity, identity::PublicKey, Multiaddr, NetworkBehaviour, PeerId, ping::{Ping, PingConfig, PingEvent}, tcp::TcpConfig, Swarm, swarm::NetworkBehaviourEventProcess, tokio_io::{AsyncRead, AsyncWrite}};
use libp2p_core::{Transport};
use libp2p_identify::{Identify, IdentifyEvent};

use crate::behaviour::{BehaviourEvent, EventEmittingBehaviour};
use std::time::Duration;

const PRIVATE_KEY: &str =
    "/O5p1cDNIyEkG3VP+LqozM+gArhSXUdWkKz6O+C6Wtr+YihU3lNdGl2iuH37ky2zsjdv/NJDzs11C1Vj0kClzQ==";

#[derive(NetworkBehaviour)]
pub struct Network<TSubstream: AsyncRead + AsyncWrite> {
    pub identify: Identify<TSubstream>,
    pub ping: Ping<TSubstream>,
    pub logging: EventEmittingBehaviour<TSubstream>
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<IdentifyEvent>
for Network<TSubstream>
{
    fn inject_event(&mut self, _event: IdentifyEvent) {}
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<PingEvent>
for Network<TSubstream>
{
    fn inject_event(&mut self, _event: PingEvent) {}
}

impl<Substream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<BehaviourEvent> for Network<Substream> {
    fn inject_event(&mut self, event: BehaviourEvent) {
        match event {
            BehaviourEvent::Connected(peer) => {
                println!("Connected: {:?}", peer);
            }
            BehaviourEvent::Disconnected(peer) => {
                println!("Disconnected: {:?}", peer);
            }
        }
    }
}

pub fn serve(port: i32) {
    // Create a random PeerId
    let mut local_key = base64::decode(PRIVATE_KEY).unwrap();
    let local_key = local_key.as_mut_slice();
    let local_key = Keypair::Ed25519(ed25519::Keypair::decode(local_key).unwrap());
    let local_peer_id = PeerId::from(local_key.public());

    println!("peer id: {}", local_peer_id);

    match local_key.public() {
        PublicKey::Ed25519(key) => println!("Public Key: {}", base64::encode(&key.encode())),
        _ => println!("Key isn't ed25519!!!!!"),
    }

    match local_key.clone() {
        identity::Keypair::Ed25519(pair) => {
            println!("PrivateKey: {}", base64::encode(&pair.encode().to_vec()))
        }
        _ => println!("Key isn't ed25519!!!!!"),
    }

    let peer_id = local_peer_id.clone();
    let transport = TcpConfig::new()
        .and_then(move |io, endpoint| {
            libp2p::core::upgrade::apply(
                io,
                plaintext::PlainText1Config {},
                endpoint,
                libp2p_core::transport::upgrade::Version::V1,
            )
        })
        .and_then(move |io, endpoint| {
            libp2p::core::upgrade::apply(io, mplex::MplexConfig::new(), endpoint, libp2p_core::transport::upgrade::Version::V1,)
        })
        .map(move |mplex, _endpoint| {
            (peer_id.clone(), mplex)
        })
        .timeout(Duration::from_secs(20));

    let mut swarm = {
        let behaviour = Network {
            identify: Identify::new("1.0.0".into(), "1.0.0".into(), local_key.public()),
            logging: EventEmittingBehaviour::new(),
            ping: Ping::new(PingConfig::with_keep_alive(PingConfig::new(), true))
        };

        Swarm::new(transport, behaviour, local_peer_id.clone())
    };

    // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
    let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port).parse().unwrap();
    Swarm::listen_on(&mut swarm, addr.clone()).unwrap();

    let mut listening = false;

    // Use tokio to drive the `Swarm`.
    tokio::run(future::poll_fn(move || -> Result<_, ()> {
        // Some comments on poll may be relevant https://github.com/libp2p/rust-libp2p/issues/1058
        loop {
            match swarm.poll().expect("Error while polling swarm") {
                Async::Ready(Some(e)) => println!("Got {:?} ready", e),
                Async::Ready(None) | Async::NotReady => {
                    if !listening {
                        if let Some(a) = Swarm::listeners(&swarm).next() {
                            println!("Listening on {}/p2p/{}", a, local_peer_id);
                            listening = true;
                        }
                    }
                    return Ok(Async::NotReady);
                }
            }
        }
    }));
}
