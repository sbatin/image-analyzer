use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::mpsc,
    thread,
};
use eyre::Result;
use tokio::sync::oneshot;

#[derive(Debug)]
enum CacheCommand<K, V> {
    Get(K, oneshot::Sender<Option<V>>),
    Set(K, V),
}

fn task_cache<K, V>(rx: mpsc::Receiver<CacheCommand<K, V>>)
where
    K: Eq + Hash + Debug,
    V: Clone,
{
    let mut cache: HashMap<K, V> = HashMap::new();

    for command in rx {
        match command {
            CacheCommand::Get(key, tx) => {
                if let Err(_) = tx.send(cache.get(&key).map(|val| val.clone())) {
                    tracing::error!("unable to send cached data for key {:?}", key);
                }
            }
            CacheCommand::Set(key, val) => {
                cache.insert(key, val);
            }
        }
    }
}

pub struct Cache<K, V> {
    commands: mpsc::Sender<CacheCommand<K, V>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Debug + Send + 'static,
    V: Clone + Send + 'static,
{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(|| task_cache(rx));
        Self { commands: tx }
    }

    pub fn get(&self, key: K) -> Result<Option<V>> {
        let (tx, rx) = oneshot::channel();
        self.commands.send(CacheCommand::Get(key, tx)).unwrap();
        let resp = rx.blocking_recv()?;
        Ok(resp)
    }

    pub fn set(&self, key: K, val: V) {
        self.commands.send(CacheCommand::Set(key, val)).unwrap();
    }
}