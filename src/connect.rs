//! The smallest example showing how to use iroh and [`iroh::Endpoint`] to connect to a remote node.
//!
//! We use the node ID (the PublicKey of the remote node), the direct UDP addresses, and the relay url to achieve a connection.
//!
//! This example uses the default relay servers to attempt to holepunch, and will use that relay server to relay packets if the two devices cannot establish a direct UDP connection.
//!
//! Run the `listen` example first (`iroh/examples/listen.rs`), which will give you instructions on how to run this example to watch two nodes connect and exchange bytes.
use std::net::SocketAddr;

use anyhow::Context;
use clap::Parser;
use iroh::{Endpoint, NodeAddr, RelayMode, RelayUrl, SecretKey};

// An example ALPN that we are using to communicate over the `Endpoint`
const EXAMPLE_ALPN: &[u8] = b"iroh-test";

#[derive(Debug, Parser)]
struct Cli {
    /// The id of the remote node.
    #[clap(long)]
    node_id: iroh::NodeId,
    /// The list of direct UDP addresses for the remote node.
    #[clap(long, value_parser, num_args = 1.., value_delimiter = ' ')]
    addrs: Vec<SocketAddr>,
    /// The url of the relay server the remote node can also be reached at.
    #[clap(long)]
    relay_url: RelayUrl,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\nconnect example!\n");
    let args = Cli::parse();
    let secret_key = SecretKey::generate(rand::rngs::OsRng);
    println!("secret key: {secret_key}");

    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .alpns(vec![EXAMPLE_ALPN.to_vec()])
        .relay_mode(RelayMode::Default)
        .bind()
        .await?;

    let me = endpoint.node_id();
    println!("node id: {me}");
    print!("node listening addresses: ");
    for local_endpoint in endpoint
        .direct_addresses()
        .initialized()
        .await
        .context("no direct addresses")?
    {
        print!("{} ", local_endpoint.addr)
    }

    let relay_url = endpoint
        .home_relay()
        .get()
        .unwrap()
        .expect("should be connected to a relay server, try calling `endpoint.local_endpoints()` or `endpoint.connect()` first, to ensure the endpoint has actually attempted a connection before checking for the connected relay server");
    println!("\nnode relay server url: {relay_url}\n");

    // build node address
    let addr = NodeAddr::from_parts(args.node_id, Some(args.relay_url), args.addrs);

    // connect to node
    let conn = endpoint.connect(addr, EXAMPLE_ALPN).await?;

    // open bi stream
    let (mut send, mut recv) = conn.open_bi().await?;

    // send message
    let message = format!("{me} is saying 'hello!'");
    send.write_all(message.as_bytes()).await?;

    // close send side
    send.finish()?;

    // receive message
    let message = recv.read_to_end(100).await?;
    let message = String::from_utf8(message)?;
    println!("received: {message}");

    // close all connections
    endpoint.close().await;
    Ok(())
}