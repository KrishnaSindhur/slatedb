use slatedb::config::{CheckpointOptions, CheckpointScope};
use slatedb::Db;
use std::{env, error::Error, sync::Arc, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // args: <PATH> [count]
    let path = env::args().nth(1).expect("usage: <PATH> [count]");
    let count: u64 = env::args().nth(2).unwrap_or_else(|| "16".into()).parse()?;

    // IMPORTANT: use the env-based loader so we write to LOCAL_PATH
    let store = slatedb::admin::load_object_store_from_env(None)?;
    let db = Db::open(path.clone(), Arc::clone(&store)).await?;

    for i in 0..count {
        db.put(format!("k{i}").as_bytes(), format!("v{i}").as_bytes())
            .await?;
        // tiny delay so seq→ts has distinct timestamps
        sleep(Duration::from_millis(5)).await;
    }

    db.create_checkpoint(CheckpointScope::All, &CheckpointOptions::default())
        .await?;
    db.close().await?;
    Ok(())
}
