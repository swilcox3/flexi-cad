use crate::*;
use futures::Future;
use serde::{Serialize, Deserialize};

pub trait Store: Send + Sync {
    fn set_store_data(&mut self, data: String);
    fn get_store_data(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestObj {
    id: RefID,
    data: String,
    point: Point3f,
    refer: Reference,
}

interfaces!(TestObj: Store, query_interface::ObjectClone, std::fmt::Debug, Data, RefPoint, Update, Position);

impl TestObj {
    pub fn new(dat: &str) -> TestObj {
        TestObj { 
            id: RefID::new_v4(), 
            data: String::from(dat), 
            point: Point3f::new(0.0, 0.0, 0.0),
            refer: Reference::nil(),
        }
    }
}

impl Data for TestObj {
    fn get_id(&self) -> &RefID {
        return &self.id;
    }

    fn set_id(&mut self, id: RefID) {
        self.id = id;
    }

    fn update(&self) -> Result<UpdateMsg, DBError> {
        Ok(UpdateMsg::Other{data: serde_json::to_value(&self).unwrap()})
    }

    fn set_data(&mut self, _data: &serde_json::Value) -> Result<(), DBError> {
        Ok(())
    }
}

impl Store for TestObj {
    fn set_store_data(&mut self, data: String) {
        self.data = data;
    }
    fn get_store_data(&self) -> String {
        self.data.clone()
    }
}

impl Update for TestObj {
    fn init(&self, deps: &DepStore) {
        deps.register_sub(&self.refer.id, self.id.clone());
    }

    fn clear_refs(&mut self) {
        self.refer = Reference::nil();
    }

    fn get_refs(&self) -> Vec<RefID> {
        let mut results = Vec::new();
        if self.refer != Reference::nil() {
            results.push(self.refer.id.clone());
        }
        results
    }

    fn update_from_refs(&mut self, ops: &ObjStore) -> Result<UpdateMsg, DBError> {
        if let Ok(pt) = ops.get_ref_point(&mut self.refer) {
            self.point = pt;
        }
        self.update()
    }
}

impl RefPoint for TestObj {
    fn get_point(&self, which: u64) -> Option<&Point3f> {
        match which {
            0 => Some(&self.point),
            _ => None 
        }
    }

    fn get_num_refs(&self) -> u64 {
        1
    }

    fn get_reference(&self, which: u64) -> Option<&Reference> {
        match which {
            0 => Some(&self.refer),
            _ => None
        }
    }

    fn set_point(&mut self, which_self: u64, pt: Point3f, other_ref: Reference) {
        match which_self {
            0 => {
                self.point = pt;
                self.refer = other_ref;
            }
            _ => ()
        }
    }
}

impl Position for TestObj {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.point = self.point + delta;
    }
}

use std::sync::{Arc, Mutex};
use crate::scheduler::Scheduler;
use tokio::timer::Delay;
use std::time::{Duration, Instant};

lazy_static! {
    static ref COUNTER: Arc<Mutex<u64>> = Arc::new(Mutex::new(1));
    static ref SET: Arc<Mutex<HashSet<String>>> = {
        let mut set = HashSet::new();
        for i in 1..100 {
            set.insert(format!("Obj {:?}", i));
        }
        Arc::new(Mutex::new(set))
    };
}
#[test]
fn test_blocking()
{
    let factory = move || {
        let clone = Arc::clone(&COUNTER);
        let mut lock = clone.lock().unwrap();
        let data = format!("Obj {:?}", lock);
        *lock = *lock + 1;
        if true {
            Ok(TestObj::new(&data))
        }
        else {
            Err(DBError::NotFound)
        }
    };
    let fut = futures::future::ok(0)
        .and_then(move |_| {
            for _ in 1..100 {
                Scheduler::spawn_fut(Scheduler::blocking(factory)
                    .map_err(|e| panic!("{:?}", e))
                    .map(|obj| {
                        let clone = Arc::clone(&SET);
                        let mut lock = clone.lock().unwrap();
                        lock.remove(&obj.data);
                    }));
            }
            Scheduler::spawn_fut(Delay::new(Instant::now() + Duration::from_secs(1))
                .map_err(|e| panic!("{:?}", e))
                .and_then(|_| {
                    let clone = Arc::clone(&SET);
                    let lock = clone.lock().unwrap();
                    assert_eq!(lock.len(), 0);
                    Ok(())
                }));
            Ok(())
        });
    Scheduler::spawn_fut(fut);
}