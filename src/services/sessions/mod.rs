use crate::prelude::*;
use bip39::{Language, Mnemonic, MnemonicType, Seed};

pub struct Session {}

pub struct SessionService {
    settings: Arc<Settings>,
    sessions: ArcMutex<HashMap<Seed, ArcRwLock<Session>>>,
}

impl SessionService {
    pub fn new(settings: Arc<Settings>) -> Self {
        Self {
            settings,
            sessions: Arc::new(Default::default()),
        }
    }

    pub async fn generate_mnemonic(&self) -> Mnemonic {
        let kind = MnemonicType::Words6;
        let lang = Language::English;
        Mnemonic::new(kind, lang)
    }
}
