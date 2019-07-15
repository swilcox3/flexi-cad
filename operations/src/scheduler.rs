use futures::Future;
use tokio::runtime::Runtime;
use crate::*;
use std::sync::Mutex;

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub struct Scheduler
{
    runtime: Runtime,
}

impl Scheduler {
    fn new() -> Scheduler {
        Scheduler {
            runtime: Runtime::new().unwrap()
        }
    }

    pub fn blocking<T, F>(mut f: F) -> impl Future<Item = T, Error = DBError>
    where
        F: FnMut() -> Result<T, DBError>,
    {
        futures::future::poll_fn(move || tokio_threadpool::blocking(&mut f))
            .map_err(|e| DBError::Other{msg: format!("{:?}", e)})
            .flatten()
    }

    pub fn block_on<T, F>(f: F) -> Result<T, DBError>
    where
        T: Send + 'static,
        F: Send + 'static + FnMut() -> Result<T, DBError>,
    {
        let mut rt = SCHEDULER.lock().unwrap();
        rt.runtime.block_on(Scheduler::blocking(f))
    }

    pub fn block_fut<F, R>(f: F) -> Result<R, DBError>
    where
        F: Send + 'static + Future<Item = R, Error = DBError>,
        R: Send + 'static
    {
        let mut rt = SCHEDULER.lock().unwrap();
        rt.runtime.block_on(f)
    }

    pub fn spawn<T, F>(f: F) 
    where
        T: Send + 'static,
        F: Send + 'static + FnMut() -> Result<T, DBError>
    {
        let mut rt = SCHEDULER.lock().unwrap();
        let fut = Scheduler::blocking(f)
            .map(|_| ())
            .map_err(|e| panic!("{:?}", e));
        rt.runtime.spawn(fut);
    }

    pub fn spawn_fut<F>(f: F)
    where
        F: Future<Item = (), Error = ()> + Send + 'static
    {
        let mut rt = SCHEDULER.lock().unwrap();
        rt.runtime.spawn(f);
    }
}

