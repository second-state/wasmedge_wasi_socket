use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wasmedge_wasi_socket::runtime::{spawn, AsyncTcpStream, Executor};

async fn stream_test() -> io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    println!("connect to 127.0.0.1:{}", port);
    let mut stream = AsyncTcpStream::connect(format!("127.0.0.1:{}", port))?;
    // send the message, remember to add '\n'
    stream.write_all(b"hello world\n").await?;

    let mut response = String::new();
    let length = stream.read_to_string(&mut response).await?;
    println!("receive: {length} \n{response}");
    Ok(())
}

fn main() -> io::Result<()> {
    let mut executor = Executor::new();
    async fn print() -> io::Result<()> {
        println!("Hello world");
        spawn(async {
            println!("dummy task!");
        });
        stream_test().await?;
        println!("finish request!");
        Ok(())
    }
    executor.block_on(print)?;
    Ok(())
}
