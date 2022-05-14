use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wasmedge_wasi_socket::executor::Executor;
use wasmedge_wasi_socket::TcpStream;

async fn stream_test() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    println!("connect to 127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    stream.set_nonblocking(true)?;

    // send the message, remember to add '\n'
    stream.write_all(b"hello world\n").await?;

    let mut response = String::new();
    let length = stream.read_to_string(&mut response).await?;
    println!("receive: {length} \n{response}");
    Ok(())
}

fn main() {
    let mut executor = Executor::new();

    executor.spawn(async {
        println!("request two!");
        if let Err(e) = stream_test().await {
            println!("{e:?}");
        }
    });
    executor.spawn(async {
        println!("request two!");
        if let Err(e) = stream_test().await {
            println!("{e:?}");
        }
    });
    executor.spawn(async {
        println!("Another block!");
    });

    executor.run();
}
