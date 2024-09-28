use madoka_auth_lib::run;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    run().await;
    Ok(())
}