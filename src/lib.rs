/*!
 * Iroh P2P library
 * 
 * A library for P2P networking functionality using Iroh.
 */

pub mod work;
pub mod sender;
pub mod receiver;
pub mod node;
 
// Re-export main types for library users
pub use work::Work;
pub use sender::Sender;
pub use receiver::Receiver;
pub use node::Node;