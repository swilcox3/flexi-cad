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
    point_2: Point3f,
    refer: Option<Reference>,
}

interfaces!(TestObj: Store, query_interface::ObjectClone, std::fmt::Debug, Data, UpdateFromRefs, HasRefs, Position, ReferTo);

impl TestObj {
    pub fn new(dat: &str) -> TestObj {
        TestObj { 
            id: RefID::new_v4(), 
            data: String::from(dat), 
            point: Point3f::new(0.0, 0.0, 0.0),
            point_2: Point3f::new(1.0, 0.0, 0.0),
            refer: None,
        }
    }
}

#[typetag::serde]
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

impl HasRefs for TestObj {
    fn init(&self, deps: &DepStore) {
        if let Some(refer) = &self.refer {
            deps.register_sub(&refer.id, self.id.clone());
        }
    }

    fn clear_refs(&mut self) {
        self.refer = None;
    }
}

impl ReferTo for TestObj {
    fn get_result(&self, which: &RefType) -> Option<RefResult> {
        match which {
            RefType::Point{..} => Some(RefResult::Point{pt: self.point}),
            _ => None 
        }
    }

    fn get_results_for_type(&self, which: &RefType) -> Vec<RefResult> {
        let mut results = Vec::new();
        match which {
            RefType::Point{..} => {
                results.push(RefResult::Point{pt: self.point});
                results.push(RefResult::Point{pt: self.point_2});
            }
            _ => ()
        }
        results
    }
}

impl UpdateFromRefs for TestObj {
    fn get_refs(&self) -> Vec<Option<Reference>> {
        let mut results = Vec::new();
        results.push(self.refer.clone());
        results
    }

    fn set_ref(&mut self, which_self: &RefType, result: &RefResult, other_ref: Reference) {
        match which_self {
            RefType::Point{which_pt} => {
                match which_pt {
                    0 => {
                        if let RefResult::Point{pt} = result {
                            self.point = *pt;
                        }
                        if let RefType::Point{..} = other_ref.ref_type {
                            self.refer = Some(other_ref);
                        }
                    }
                    1 => {
                        if let RefResult::Point{pt} = result {
                            self.point_2 = *pt;
                        }
                        if let RefType::Point{..} = other_ref.ref_type {
                            self.refer = Some(other_ref);
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }

    fn update_from_refs(&mut self, results: &Vec<Option<RefResult>>) -> Result<UpdateMsg, DBError> {
        if let Some(refer) = results.get(0) {
            if let Some(RefResult::Point{pt}) = refer {
                self.point = *pt;
            }
        }
        if let Some(refer) = results.get(1) {
            if let Some(RefResult::Point{pt}) = refer {
                self.point_2 = *pt;
            }
        }
        self.update()
    }
}

impl Position for TestObj {
    fn move_obj(&mut self, delta: &Vector3f) {
        self.point = self.point + delta;
        self.point_2 = self.point + delta;
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