use std::net::SocketAddr;

use nix::sys::socket;
use nix::sys::socket::sockopt::IpTransparent;
use tokio::net::TcpSocket;

const PORT: u16 = 15006;
const LISTENER_BACKLOG: u32 = 65535;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listen_addr = format!("0.0.0.0:{}", PORT).parse().unwrap();
    println!("Listening on: {}", listen_addr);
    let socket = TcpSocket::new_v4()?;

    #[cfg(any(target_os = "linux"))]
    socket::setsockopt(&socket, IpTransparent, &true)?;

    socket.bind(listen_addr)?;
    let listener = socket.listen(LISTENER_BACKLOG)?;

    while let Ok((mut downstream_conn, client_addr)) = listener.accept().await {
        println!("accept new connection, peer[{:?}]->local[{:?}]", downstream_conn.peer_addr()?, downstream_conn.local_addr()?);

        let jh = tokio::spawn({
            let client_real_ip = downstream_conn.peer_addr()?.ip();
            let upstream_addr: SocketAddr = format!("127.0.0.1:{}", downstream_conn.local_addr()?.port()).parse()?;
            async move {
                println!("start connect to upstream: {}", upstream_addr);
                let socket = TcpSocket::new_v4()?;
                // #[cfg(any(target_os = "linux"))]
                // socket::setsockopt(&socket, IpTransparent, &true)?;

                let bind_addr = SocketAddr::new(client_real_ip, 0);
                match socket.bind(bind_addr) {
                    Ok(_) => {
                        println!("bind to: {} success", bind_addr);
                    }
                    Err(err) => {
                        println!("bind to: {} failed, err: {:?}", bind_addr, err);
                        return Err(err.into());
                    }
                };

                let mut upstream_conn = socket.connect(upstream_addr).await?;

                println!("connected to upstream, local[{:?}]->peer[{:?}]", upstream_conn.local_addr()?, upstream_conn.peer_addr()?);
                tokio::io::copy_bidirectional(&mut downstream_conn, &mut upstream_conn).await?;
                Ok::<(), anyhow::Error>(())
            }
        });
        let join_result = jh.await;
        println!("connection closed: {:?}", join_result);
    }

    Ok(())
}
