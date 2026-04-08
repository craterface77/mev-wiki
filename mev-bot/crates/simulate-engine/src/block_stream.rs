use std::sync::Arc;

use alloy::consensus::BlockHeader;
use alloy::providers::{Provider, ProviderBuilder, RootProvider, WsConnect};
use alloy::pubsub::Subscription;
use alloy::rpc::types::Header;
use futures::StreamExt;
use tracing::{info, warn};

use crate::pool_state::PoolStateStore;

pub struct BlockStream {
    ws_url:     String,
    pool_store: Arc<PoolStateStore>,
}

impl BlockStream {
    pub fn new(ws_url: String, pool_store: Arc<PoolStateStore>) -> Self {
        Self { ws_url, pool_store }
    }

    pub async fn run(self) {
        self.run_with_callback(|_| {}).await;
    }

    pub async fn run_with_callback<F>(self, on_block: F)
    where
        F: Fn(u64) + Send + 'static,
    {
        loop {
            match self.run_inner(&on_block).await {
                Ok(())  => info!("BlockStream: stream ended, reconnecting..."),
                Err(e)  => warn!("BlockStream: {e:#}, reconnecting..."),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    async fn run_inner<F>(&self, on_block: &F) -> anyhow::Result<()>
    where
        F: Fn(u64),
    {
        let ws = WsConnect::new(self.ws_url.clone());
        let provider: RootProvider =
            ProviderBuilder::new()
                .disable_recommended_fillers()
                .connect_ws(ws)
                .await?;

        let sub: Subscription<Header> = provider.subscribe_blocks().await?;
        let mut stream = sub.into_stream();

        info!("BlockStream: subscribed");

        while let Some(header) = stream.next().await {
            let number   = header.number();
            let base_fee = header.base_fee_per_gas().unwrap_or(0);

            self.pool_store.set_block_number(number);
            self.pool_store.set_base_fee(base_fee);

            info!(
                "block #{number} base_fee={:.2} gwei",
                base_fee as f64 / 1e9
            );

            on_block(number);
        }

        Ok(())
    }
}
