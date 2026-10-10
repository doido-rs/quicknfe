use std::collections::HashMap;
use std::sync::RwLock;

use crate::models::StoredDps;

#[derive(Default)]
pub struct DpsStore {
    inner: RwLock<HashMap<String, StoredDps>>,
}

impl DpsStore {
    pub fn insert(&self, record: StoredDps) {
        let mut guard = self.inner.write().expect("store lock");
        guard.insert(record.id_dps.clone(), record);
    }

    pub fn get(&self, id: &str) -> Option<StoredDps> {
        self.inner.read().expect("store lock").get(id).cloned()
    }
}
