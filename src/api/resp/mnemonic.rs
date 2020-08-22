use bip39::Mnemonic;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MnemonicResp {
    pub phrase: String,
}

impl From<Mnemonic> for MnemonicResp {
    fn from(m: Mnemonic) -> Self {
        Self {
            phrase: m.phrase().to_string(),
        }
    }
}
