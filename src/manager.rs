use std::{
    collections::HashMap,
    hash::Hash,
};
use tokio::{
    task::{self, JoinHandle},
    sync::watch,
};

pub enum TaskResponse<P, R> {
    Pending(P),
    Completed(R),
}

pub struct TaskManager<K, P, R> {
    tasks: HashMap<K, (JoinHandle<R>, watch::Receiver<P>)>,
}

impl<K, P, R> TaskManager<K, P, R>
where
    K: Eq + Hash,
    P: Send + Sync + 'static,
    R: Send + 'static,
{
    pub fn new() -> Self {
        Self { tasks: HashMap::new() }
    }

    pub fn submit<F>(&mut self, key: K, f: F) -> &mut (JoinHandle<R>, watch::Receiver<P>)
    where
        F: FnOnce(watch::Sender<P>) -> R + Send + 'static,
        P: Default,
    {
        self.tasks.entry(key).or_insert_with(|| {
            let (tx, rx) = watch::channel(Default::default());
            let join_handle = task::spawn_blocking(|| f(tx));
            (join_handle, rx)
        })
    }

    pub async fn poll(&mut self, key: &K) -> Option<TaskResponse<P, R>>
    where
        P: Copy
    {
        let (key, (join_handle, mut rx)) = self.tasks.remove_entry(key)?;
        Some(if let Ok(_) = rx.changed().await {
            let progress = *rx.borrow();
            // still in progress: put handles back to tasks
            self.tasks.insert(key, (join_handle, rx));
            TaskResponse::Pending(progress)
        } else {
            let result = join_handle.await.unwrap();
            TaskResponse::Completed(result)
        })
    }

    pub fn progress(&self, key: &K) -> Option<watch::Receiver<P>> {
        let (_, rx) = self.tasks.get(key)?;
        Some(rx.clone())
    }
}