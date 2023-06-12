use anyhow::Result;
use libp2p::futures::StreamExt;
use libp2p::swarm::{keep_alive, NetworkBehaviour, SwarmBuilder, SwarmEvent};
use libp2p::{identity, ping, Multiaddr, PeerId};

pub async fn test() -> Result<()> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {local_peer_id:?}");

    let transport = libp2p::development_transport(local_key).await?;

    let behaviour = Behaviour::default();

    let mut swarm = SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id).build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    if let Some(addr) = Some("/dnsaddr/api.drand.sh") {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }

    loop {
      match swarm.select_next_some().await {
          SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
          SwarmEvent::Behaviour(event) => println!("{event:?}"),
          _ => {}
      }
    }

    Ok(())
}

#[derive(NetworkBehaviour, Default)]
struct Behaviour {
    keep_alive: keep_alive::Behaviour,
    ping: ping::Behaviour,
}