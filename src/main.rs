use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Recv, Request, Response};
use std::env;
use tokio::net::TcpListener;

async fn get_listener() -> TcpListener {
    #[cfg(not(target_os = "wasi"))]
    {
        use std::net::SocketAddr;
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        TcpListener::bind(addr).await.unwrap()
    }
    #[cfg(target_os = "wasi")]
    {
        use std::os::wasi::io::FromRawFd;
        let std_listener = unsafe { std::net::TcpListener::from_raw_fd(3) };
        std_listener.set_nonblocking(true).unwrap();
        TcpListener::from_std(std_listener).unwrap()
    }
}

fn platform() -> String {
    let mut name = env::consts::ARCH.to_string();
    if env::consts::OS.len() > 0 {
        name = format!("{}-{}", name, env::consts::OS);
    }
    name
}

async fn hello(_x: Request<Recv>) -> Result<Response<String>, String> {
    Ok(Response::new(String::from(format!(
        "hello from {}!",
        platform()
    ))))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // Adapted from https://hyper.rs/guides/server/hello-world/
    let tcp_conn = get_listener().await;

    let serv = service_fn(hello);

    loop {
        let (tcp_stream, _) = tcp_conn.accept().await?;
        tokio::task::spawn(async move {
            let mut h = Http::new();
            h.http1_only(true);
            h.http1_keep_alive(true);

            let conn = h.serve_connection(tcp_stream, serv);

            if let Err(http_err) = conn.await {
                eprintln!("Error while serving HTTP connection: {}", http_err);
            }
        });
    }
}
