use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wasmedge_wasi_socket::runtime::{spawn, AsyncTcpStream, Executor};

async fn stream_test() -> io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("1235".to_string());
    println!("Connect to 127.0.0.1:{}", port);
    let mut stream = AsyncTcpStream::connect(format!("127.0.0.1:{}", port))?;
    // send the message, remember to add '\n'
    stream.write_all(b"hello world\n").await?;
    stream.flush().await?;
    println!("Flush.");
    let mut response = String::new();
    let length = stream.read_to_string(&mut response).await?;
    println!("receive: {length}\n{response}");
    assert_eq!(response, "Hello UDP Client! I received a message from you!");
    Ok(())
}

fn main() -> io::Result<()> {
    let mut executor = Executor::new();
    async fn connect() -> io::Result<()> {
        println!("Before connecting ...");
        spawn(async {
            println!("Dummy task!");
        });
        stream_test().await?;
        println!("Finish request!");
        Ok(())
    }
    executor.block_on(connect)?;
    Ok(())
}
