use std::{net::SocketAddr, time::Duration};

use argh::FromArgs;
use async_trait::async_trait;
use melnet2::{wire::tcp::TcpBackhaul, Backhaul};
use melnet2_bench::{BenchProtocol, BenchService};

#[derive(FromArgs)]
/// Runs a benchmarking server over the melnet2 transport network.
struct ServerArgs {
    /// where to listen (ip:port)
    #[argh(option)]
    listen: SocketAddr,
}

struct BenchServerImpl;

#[async_trait]
impl BenchProtocol for BenchServerImpl {
    async fn hello_world(&self) -> String {
        "hello world".to_string()
    }

    async fn delayed_echo(&self, s: String, secs: u64) -> String {
        smol::Timer::after(Duration::from_secs(secs)).await;
        s
    }
}

fn main() {
    let args: ServerArgs = argh::from_env();
    smolscale::block_on(async move {
        let bhaul = TcpBackhaul::new();
        bhaul
            .start_listen(
                args.listen.to_string().into(),
                BenchService(BenchServerImpl),
            )
            .await
            .unwrap();
        eprintln!("listening on {}", args.listen);
        smol::future::pending().await
    })
}
