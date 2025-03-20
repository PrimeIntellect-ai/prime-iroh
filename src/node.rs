use anyhow::{Context, Result};
use iroh::{NodeId, NodeAddr, Endpoint, endpoint::{Connection, SendStream, RecvStream}};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::runtime::Runtime;

const ALPN: &[u8] = b"hello-world";

pub struct IrohWork<T> {
    runtime: Arc<Runtime>,
    handle: JoinHandle<Result<T>>,
}

impl<T> IrohWork<T> {
    pub fn wait(self) -> Result<T> {
        self.runtime.block_on(self.handle)?
    }
}

pub struct IrohNode {
    node_id: NodeId,
    runtime: Arc<Runtime>,
    endpoint: Endpoint,
    num_streams: usize,
    send_connection: Option<Connection>,
    recv_connection: Option<Connection>,
    send_streams: Option<Vec<Arc<Mutex<SendStream>>>>,
    recv_streams: Option<Vec<Arc<Mutex<RecvStream>>>>,
}

impl IrohNode {
    pub fn new(num_streams: usize) -> Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        let (endpoint, node_id) = runtime.block_on(async {
            let endpoint = Endpoint::builder()
                .discovery_n0()
                .alpns(vec![ALPN.to_vec()])
                .bind()
                .await?;
            let node_id = endpoint.node_id();
            
            Ok::<(Endpoint, NodeId), anyhow::Error>((endpoint, node_id))
        })?;

        Ok(Self { 
            node_id,
            runtime,
            endpoint,
            num_streams,
            send_connection: None,
            recv_connection: None,
            send_streams: None,
            recv_streams: None,
        })
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn listen(&mut self) -> Result<()> {
        if self.recv_connection.is_some() {
            return Err(anyhow::anyhow!("Connection already established"));
        }

        let (connection, recv_streams) = self.runtime.block_on(async {
            let connection = self.endpoint.accept().await.context("no incoming connection")?.await?;
            let mut recv_streams = Vec::with_capacity(self.num_streams);
            for _ in 0..self.num_streams {
                let recv_stream = connection.accept_uni().await?;
                recv_streams.push(Arc::new(Mutex::new(recv_stream)));
            }
            Ok::<(Connection, Vec<Arc<Mutex<RecvStream>>>), anyhow::Error>((connection, recv_streams))
        })?;

        self.recv_connection = Some(connection);
        self.recv_streams = Some(recv_streams);
        Ok(())
    }

    pub fn connect(&mut self, node_id_str: String) -> Result<()> {
        if self.send_connection.is_some() {
            return Err(anyhow::anyhow!("Connection already established"));
        }

        let (connection, send_streams) = self.runtime.block_on(async {
            let addr = self.get_node_addr(node_id_str)?;
            let connection = self.endpoint.connect(addr, ALPN).await?;
            let mut send_streams = Vec::with_capacity(self.num_streams);
            for _ in 0..self.num_streams {
                let send_stream = Arc::new(Mutex::new(connection.open_uni().await?));
                send_streams.push(send_stream);
            }
            Ok::<(Connection, Vec<Arc<Mutex<SendStream>>>), anyhow::Error>((connection, send_streams))
        })?;

        self.send_connection = Some(connection);
        self.send_streams = Some(send_streams);
        Ok(())
    }

    pub fn isend(&mut self, msg: Vec<u8>, tag: usize) -> Result<IrohWork<()>> {
        if self.send_streams.is_none() {
            return Err(anyhow::anyhow!("No send streams available - did you call connect()?"));
        }
        
        let stream = self.send_streams.as_ref().unwrap()[tag].clone();
        let handle = self.runtime.spawn(async move {
            let mut stream = stream.lock().await;
            let size = msg.len() as u32;
            stream.write_all(&size.to_le_bytes()).await?;
            stream.write_all(&msg).await?;
            Ok(())
        });
        Ok(IrohWork {
            runtime: self.runtime.clone(),
            handle,
        })
    }

    pub fn send(&mut self, msg: Vec<u8>, tag: usize) -> Result<()> {
        self.isend(msg, tag)?.wait()
    }

    pub fn irecv(&mut self, tag: usize) -> Result<IrohWork<Vec<u8>>> {
        if self.recv_streams.is_none() {
            return Err(anyhow::anyhow!("No receive streams available - did you call accept()?"));
        }
        
        let stream = self.recv_streams.as_ref().unwrap()[tag].clone();
        let handle = self.runtime.spawn(async move {
            let mut stream = stream.lock().await;
            // Read the size of the message
            let mut size = [0; 4];
            stream.read_exact(&mut size).await?;
            let size = u32::from_le_bytes(size) as usize;

            // Read the message
            let mut msg = vec![0; size];
            stream.read_exact(&mut msg).await?;
            Ok(msg)
        });
        Ok(IrohWork {
            runtime: self.runtime.clone(),
            handle,
        })
    }

    pub fn recv(&mut self, tag: usize) -> Result<Vec<u8>> {
        self.irecv(tag)?.wait()
    }

    pub fn close(&mut self) -> Result<()> {
        self.runtime.block_on(async {
            // Close send streams
            if let Some(streams) = &self.send_streams {
                for stream in streams {
                    let mut stream = stream.lock().await;
                    stream.stopped().await?;
                }
            }
            // Close receive streams
            if let Some(streams) = &self.recv_streams {
                for stream in streams {
                    let mut stream = stream.lock().await;
                    stream.stop(0u32.into())?;
                }
            }
            // Close connection if it exists
            if let Some(send_connection) = &self.send_connection {
                send_connection.close(0u32.into(), b"close");
            }
            if let Some(recv_connection) = &self.recv_connection {
                recv_connection.closed().await;
            }
            self.endpoint.close().await;
            Ok(())
        })
    }

    fn get_node_addr(&self, node_id_str: String) -> Result<NodeAddr> {
        let bytes = hex::decode(node_id_str)?;
        let node_id = NodeId::from_bytes(bytes.as_slice().try_into()?)?;
        Ok(NodeAddr::new(node_id))
    }
}

pub fn main() -> Result<()> {
    use std::process::Command;
    use std::env;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    let num_new_tokens  = 2;
    let num_micro_batches = 2;

    // Check if we're the child process
    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "receiver" => {
                // Create receiver
                let mut receiver = IrohNode::new(num_micro_batches)?;
                let node_id = hex::encode(receiver.node_id().as_bytes());
                fs::write("node_id.tmp", &node_id)?;

                // Wait for connection
                receiver.listen()?;

                // Receive messages
                for token_idx in 0..num_new_tokens {
                    for micro_batch_idx in 0..num_micro_batches {
                        let recv_handle = receiver.irecv(micro_batch_idx)?;
                        let _ = recv_handle.wait();
                        println!("Received token idx {} micro batch idx {}", token_idx, micro_batch_idx);
                    }
                }
                receiver.close()?;

                return Ok(());
            },
            "sender" => {
                // Wait for and read node ID
                for _ in 0..10 {
                    if let Ok(node_id_str) = fs::read_to_string("node_id.tmp") {
                        let mut sender = IrohNode::new(num_micro_batches)?;
                        sender.connect(node_id_str)?;

                        // Send messages
                        for token_idx in 0..num_new_tokens {
                            let mut handles = Vec::new();
                            for micro_batch_idx in 0..num_micro_batches {
                                // Send message
                                println!("Sending token idx {} micro batch idx {}", token_idx, micro_batch_idx);
                                let msg = vec![token_idx as u8; 1024 * 1024 * 50];
                                let send_handle = sender.isend(msg, micro_batch_idx)?;
                                handles.push(send_handle);
                            }
                            for handle in handles {
                                let _ = handle.wait();
                            }
                        }

                        // Close the sender
                        sender.close()?;

                        return Ok(());
                    }
                    thread::sleep(Duration::from_secs(1));
                }
                return Err(anyhow::anyhow!("Timeout waiting for node ID"));
            },
            _ => return Err(anyhow::anyhow!("Invalid argument")),
        }
    }

    // We're the parent process - spawn both processes
    let receiver = Command::new(env::current_exe()?)
        .arg("receiver")
        .spawn()?;

    thread::sleep(Duration::from_secs(1));

    let sender = Command::new(env::current_exe()?)
        .arg("sender")
        .spawn()?;

    receiver.wait_with_output()?;
    sender.wait_with_output()?;

    let _ = fs::remove_file("node_id.tmp");
    Ok(())
}