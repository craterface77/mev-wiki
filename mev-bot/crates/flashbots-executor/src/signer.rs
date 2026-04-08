#[allow(unused_imports)]
use k256::ecdsa::{signature::hazmat::PrehashSigner, SigningKey};
#[allow(unused_imports)]
use k256::elliptic_curve::sec1::ToEncodedPoint;
use sha3::{Digest, Keccak256};

pub struct FlashbotsSigner {
    key:     SigningKey,
    address: String,
}

impl FlashbotsSigner {
    pub fn from_hex(private_key_hex: &str) -> anyhow::Result<Self> {
        let key_bytes = hex::decode(private_key_hex.trim_start_matches("0x"))?;
        let key = SigningKey::from_slice(&key_bytes)?;
        let address = derive_address(&key);
        Ok(Self { key, address })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    /// X-Flashbots-Signature: EIP-191 personal_sign(keccak256(body))
    pub fn sign_body(&self, body: &[u8]) -> anyhow::Result<String> {

        let body_hash = Keccak256::digest(body);
        let prefix    = b"\x19Ethereum Signed Message:\n32";
        let mut msg   = Vec::with_capacity(prefix.len() + 32);
        msg.extend_from_slice(prefix);
        msg.extend_from_slice(body_hash.as_slice());
        let msg_hash = Keccak256::digest(&msg);

        let (sig, recid) = self.key.sign_prehash_recoverable(msg_hash.as_slice())?;
        let sig_bytes = sig.to_bytes();
        let v = 27u8 + recid.to_byte();

        let mut full_sig = Vec::with_capacity(65);
        full_sig.extend_from_slice(&sig_bytes);
        full_sig.push(v);

        Ok(format!("{}:0x{}", self.address, hex::encode(full_sig)))
    }
}

fn derive_address(key: &SigningKey) -> String {
    let pubkey       = key.verifying_key();
    let encoded      = pubkey.to_encoded_point(false);
    let pubkey_bytes = &encoded.as_bytes()[1..];
    let hash         = Keccak256::digest(pubkey_bytes);
    format!("0x{}", hex::encode(&hash[12..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_derivation() {
        let signer = FlashbotsSigner::from_hex(
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        ).unwrap();
        assert_eq!(
            signer.address().to_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    #[test]
    fn test_sign_body_length() {
        let signer = FlashbotsSigner::from_hex(
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        ).unwrap();
        let header = signer.sign_body(b"{}").unwrap();
        assert!(header.contains(':'));
        let parts: Vec<&str> = header.splitn(2, ':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 132);
    }
}
