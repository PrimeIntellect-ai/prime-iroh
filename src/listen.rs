//! $ cargo run --bin listen
use std::time::Duration;

use iroh::{endpoint::ConnectionError, Endpoint, RelayMode, SecretKey};

// ALPN to use for the connection
const EXAMPLE_ALPN: &[u8] = b"iroh-test";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\nlisten example!\n");
    // generate secret key
    let secret_key = SecretKey::generate(rand::rngs::OsRng);
    println!("secret key: {secret_key}");

    // build endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .alpns(vec![EXAMPLE_ALPN.to_vec()])
        .relay_mode(RelayMode::Default)
        .bind()
        .await?;

    // print local addresses
    let me = endpoint.node_id();
    println!("node id: {me}");
    print!("node listening addresses: ");
    let node_addr = endpoint.node_addr().await?;
    let local_addrs = node_addr
        .direct_addresses
        .into_iter()
        .map(|addr| {
            let addr = addr.to_string();
            print!("{addr}\t");
            addr
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!();
    let relay_url = node_addr
        .relay_url
        .expect("Should have a relay URL, assuming a default endpoint setup.");
    println!("node relay server url: {relay_url}");

    // print instructions for connecting
    println!(
        "\ncargo run --bin connect -- --node-id {me} --addrs \"{local_addrs}\" --relay-url {relay_url}\n"
    );

    // accept incoming connections, returns a normal QUIC connection
    while let Some(incoming) = endpoint.accept().await {
        let mut connecting = match incoming.accept() {
            Ok(connecting) => connecting,
            Err(err) => {
                println!("incoming connection failed: {err:#}");
                // we can carry on in these cases:
                // this can be caused by retransmitted datagrams
                continue;
            }
        };
        let alpn = connecting.alpn().await?;
        let conn = connecting.await?;
        let node_id = conn.remote_node_id()?;
        println!(
            "new connection from {node_id} with ALPN {}",
            String::from_utf8_lossy(&alpn),
        );

        // spawn a task to handle reading and writing off of the connection
        tokio::spawn(async move {
            // accept a bi-directional QUIC connection
            // use the `quinn` APIs to send and recv content
            let (mut send, mut recv) = conn.accept_bi().await?;
            println!("accepted bi stream, waiting for data...");
            let message = recv.read_to_end(100).await?;
            let message = String::from_utf8(message)?;
            println!("received: {message}");

            let message = format!("hi! you connected to {me}. bye bye");
            send.write_all(message.as_bytes()).await?;
            // call `finish` to close the connection gracefully
            send.finish()?;

            // We sent the last message, so wait for the client to close the connection once
            // it received this message.
            let res = tokio::time::timeout(Duration::from_secs(3), async move {
                let closed = conn.closed().await;
                if !matches!(closed, ConnectionError::ApplicationClosed(_)) {
                    println!("node {node_id} disconnected with an error: {closed:#}");
                }
            })
            .await;
            if res.is_err() {
                println!("node {node_id} did not disconnect within 3 seconds");
            }
            Ok::<_, anyhow::Error>(())
        });
    }
    // stop with SIGINT (ctrl-c)

    Ok(())
}