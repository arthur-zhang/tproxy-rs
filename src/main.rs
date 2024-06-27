use nix::sys::socket;
use nix::sys::socket::sockopt::IpTransparent;
use tokio::net::{TcpListener, TcpSocket};

const PORT: u16 = 15006;
const LISTENER_BACKLOG: u32 = 65535;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let listen_addr = format!("0.0.0.0:{}", PORT).parse().unwrap();
    println!("Listening on: {}", listen_addr);
    let socket = TcpSocket::new_v4()?;

    #[cfg(any(target_os = "linux"))]
    socket::setsockopt(&socket, IpTransparent, &true)?;

    socket.bind(listen_addr)?;
    let listener =  socket.listen(LISTENER_BACKLOG)?;

    while let Ok((downstream_conn, client_addr)) = listener.accept().await {
        println!("client addr: {:?}", client_addr);
        tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;
    }

    Ok(())
}
