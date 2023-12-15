use futures::{future::Fuse, pin_mut, Future, FutureExt, SinkExt, StreamExt};
use gloo_utils::format::JsValueSerdeExt;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::DomException;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wasm_bindgen::JsValue;
use yew::platform::time::sleep;
use yew_agent::prelude::*;
use yewdux::log::{log, Level};

/// IndexedDB agent errors.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Error {
    /// Error during IndexedDB operation.
    IndexedDb { msg: String },
    /// Error during serde serialization.
    Serialization { msg: String },
    /// Error during serde deserialization.
    Deserialization { msg: String },
}
impl From<DomException> for Error {
    fn from(value: DomException) -> Self {
        Self::IndexedDb {
            msg: format!("{:?}", value),
        }
    }
}

pub type Queue<T> = Arc<Mutex<QueueInner<T>>>;
pub type QueueInner<T> = BTreeMap<DatabaseObjectPointer, Request<T>>;
pub type Job<T> = Arc<Mutex<JobInner<T>>>;
pub type JobInner<T> = Option<Request<T>>;
pub type Handle<'a> = Pin<&'a mut Fuse<dyn Future<Output = Result<(), Error>>>>;

#[reactor]
pub async fn IndexedDbReactor<T>(mut scope: ReactorScope<Request<T>, Response>)
where
    T: 'static + Unpin + Serialize,
{
    // Worker "state".
    let queue: Queue<T> = Arc::new(Mutex::new(BTreeMap::default()));
    let job: Job<T> = Arc::new(Mutex::new(None));
    let mut status = QueueStatus::default();

    // Create a job handle and pin it such that we can change it in the loop.
    let handle = handle_job(Arc::clone(&job)).fuse();
    pin_mut!(handle);

    loop {
        // Select between receiving, responding or waiting a little.
        futures::select! {
            // Receive a message.
            req = scope.next() => {
                if let (Some(req), Ok(mut queue)) = (req, queue.lock()) {
                    match &req {
                        Request::Put(put) => {
                            queue.insert(put.pointer.clone(), req);
                        }
                    }
                }
            },
            // Handle a job (or wait a little).
            res = handle => {
                if let Err(job_err) = res {
                    if let Err(send_err) = scope.send(Response::Error(job_err)).await {
                        log!(Level::Error, "{:?}", send_err);
                    }
                }
            }
            _ = sleep(Duration::from_millis(100)).fuse() => {}
        };

        // Update the queue's status and create a new job handle if we should.
        if status.update(Arc::clone(&queue), Arc::clone(&job)) {
            handle.set(handle_job(Arc::clone(&job)).fuse());
        }

        // Send our most recent status.
        if let Err(e) = scope.send(Response::QueueStatus(status.clone())).await {
            log!(Level::Error, "{:?}", e);
        }
    }
}

pub async fn handle_job<T: Serialize>(job: Job<T>) -> Result<(), Error> {
    // Acquire the lock on the job, this prevents other calls to the active job.
    if let Some(mut job) = job.try_lock().ok() {
        // Handle a request if we have one,
        // Clear out the job value, regardless of the result.
        sleep(Duration::from_millis(100)).await;
        if let Some(request) = job.take() {
            return match request {
                Request::Put(req) => save(req.pointer, req.data).await,
            };
        }
    }
    Ok(())
}

/// Types of requests for the worker.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request<T> {
    Put(PutRequest<T>),
}
impl<T: Clone> Request<T> {
    /// Create a PUT request.
    pub fn put(database: String, object: String, value: T) -> Self {
        Self::Put(PutRequest::new(database, object, value))
    }
}

/// An IndexedDB PUT request.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutRequest<T> {
    /// Pointer to the database object.
    pub pointer: DatabaseObjectPointer,
    /// Data to put.
    pub data: Box<T>,
}
impl<T> PutRequest<T> {
    pub fn new(database: String, object: String, value: T) -> Self {
        Self {
            pointer: DatabaseObjectPointer { database, object },
            data: Box::new(value),
        }
    }
}

/// Types of worker responses.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// Active job and current waiting job per pointer.
    QueueStatus(QueueStatus),
    /// Error during job execution.
    Error(Error),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct QueueStatus {
    /// Active job.
    pub active: Option<DatabaseObjectPointer>,
    /// Which pointers have remaining jobs.
    pub waiting: BTreeSet<DatabaseObjectPointer>,
}
impl QueueStatus {
    pub fn update<T>(&mut self, queue: Queue<T>, job: Job<T>) -> bool {
        let mut new_job = false;
        if let (Ok(mut queue), Ok(mut job)) = (queue.try_lock(), job.try_lock()) {
            if job.is_none() {
                if let Some((key, value)) = queue.pop_first() {
                    *job = Some(value);
                    self.active = Some(key);
                    new_job = true;
                } else {
                    self.active = None;
                }
            }
        }
        if let Ok(queue) = queue.try_lock() {
            self.waiting = queue.keys().cloned().collect();
        }
        new_job
    }
}

/// Pointer to an object in a database.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DatabaseObjectPointer {
    /// Name to the IndexedDB database.
    pub database: String,
    /// Name of the object in the store.
    pub object: String,
}
impl DatabaseObjectPointer {
    pub fn new(database: String, object: String) -> Self {
        Self { database, object }
    }
}

/// Save the value to the given database object pointer.
pub async fn save<T: Serialize>(pointer: DatabaseObjectPointer, value: T) -> Result<(), Error> {
    let db = database(pointer.database.clone()).await?;

    let tx = db.transaction_on_one_with_mode(&pointer.database, IdbTransactionMode::Readwrite)?;
    let store = tx.object_store(&pointer.database)?;

    let value =
        <JsValue as JsValueSerdeExt>::from_serde(&value).map_err(|e| Error::Serialization {
            msg: format!("{:?}", e),
        })?;

    store.put_key_val_owned(pointer.object, &value)?;

    Ok(())
}

/// Load a value from the given pointer.
pub async fn load<T: for<'de> Deserialize<'de>>(
    pointer: DatabaseObjectPointer,
) -> Result<Option<T>, Error> {
    let db = database(pointer.database.clone()).await?;

    let tx = db.transaction_on_one(&pointer.database)?;
    let store = tx.object_store(&pointer.database)?;

    let value: Option<JsValue> = store.get_owned(pointer.object)?.await?;
    log!(Level::Info, "got value {:?}", &value);

    let value = match value {
        Some(v) => {
            let de = <JsValue as JsValueSerdeExt>::into_serde(&v).map_err(|e| {
                Error::Deserialization {
                    msg: format!("{:?}", e),
                }
            })?;
            Some(de)
        }
        None => None,
    };

    Ok(value)
}

/// Get the database with this name.
pub async fn database(name: String) -> Result<IdbDatabase, Error> {
    let mut db_req = IdbDatabase::open(&name)?;

    let on_upgrade = {
        let store_name = name.clone();
        Some(move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Check if the object store exists; create it if it doesn't
            if evt
                .db()
                .object_store_names()
                .find(|n| n == &store_name)
                .is_none()
            {
                evt.db().create_object_store(&store_name)?;
            }
            Ok(())
        })
    };
    db_req.set_on_upgrade_needed(on_upgrade);

    Ok(db_req.await?)
}
