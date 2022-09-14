use async_trait::async_trait;
use nanorpc::nanorpc_derive;

/// The common protocol between server and client
#[nanorpc_derive]
#[async_trait]
pub trait BenchProtocol {
    /// Returns the string "hello world".
    async fn hello_world(&self) -> String;

    /// Waits the given number of seconds, then returns the given string.
    async fn delayed_echo(&self, s: String, secs: u64) -> String;
}