use madoka_auth_lib::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await?;
    Ok(())
}
