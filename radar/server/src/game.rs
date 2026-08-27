use std::sync::Arc;

use shared::data::Data;
use tokio::sync::RwLock;

pub type Games = Arc<RwLock<Data>>;
