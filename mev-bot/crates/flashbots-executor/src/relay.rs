use reqwest::Client;
use tracing::{debug, warn};

use crate::bundle::FlashbotsBundle;
use crate::signer::FlashbotsSigner;

pub const FLASHBOTS_RELAY: &str = "https://relay.flashbots.net";

pub struct FlashbotsRelay {
    client: Client,
    signer: FlashbotsSigner,
    url:    String,
}

impl FlashbotsRelay {
    pub fn new(signer: FlashbotsSigner) -> Self {
        Self::with_url(signer, FLASHBOTS_RELAY.to_string())
    }

    pub fn with_url(signer: FlashbotsSigner, url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("failed to build reqwest client");
        Self { client, signer, url }
    }

    pub async fn send_bundle(&self, bundle: &FlashbotsBundle) -> anyhow::Result<String> {
        let body       = bundle.to_json_body();
        let body_bytes = serde_json::to_vec(&body)?;
        let auth       = self.signer.sign_body(&body_bytes)?;

        debug!("sending bundle block={} txs={}", bundle.block_number, bundle.txs.len());

        let resp = self.client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("X-Flashbots-Signature", &auth)
            .body(body_bytes)
            .send()
            .await?;

        let status = resp.status();
        let text   = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("relay HTTP {status}: {text}");
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)?;
        if let Some(err) = parsed.get("error") {
            anyhow::bail!("relay error: {err}");
        }

        Ok(parsed["result"]["bundleHash"]
            .as_str()
            .unwrap_or("unknown")
            .to_string())
    }

    pub async fn simulate_bundle(
        &self,
        bundle: &FlashbotsBundle,
        state_block: u64,
    ) -> anyhow::Result<serde_json::Value> {
        let txs: Vec<String> = bundle.txs
            .iter()
            .map(|t| format!("0x{}", hex::encode(&t.raw)))
            .collect();

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_callBundle",
            "params": [{
                "txs": txs,
                "blockNumber": format!("0x{:x}", bundle.block_number),
                "stateBlockNumber": format!("0x{:x}", state_block),
            }]
        });

        let body_bytes = serde_json::to_vec(&body)?;
        let auth       = self.signer.sign_body(&body_bytes)?;

        let resp = self.client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("X-Flashbots-Signature", &auth)
            .body(body_bytes)
            .send()
            .await?;

        let status = resp.status();
        let text   = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("relay simulate HTTP {status}: {text}");
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)?;
        if let Some(err) = parsed.get("error") {
            warn!("simulate_bundle error: {err}");
            anyhow::bail!("relay simulate error: {err}");
        }

        Ok(parsed["result"].clone())
    }
}
