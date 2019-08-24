use crate::prelude::*;
use ccl::dhashmap::DHashMap;

pub struct DependencyManager {
    pub_subs: DHashMap<RefID, HashSet<RefID>>
}

impl DependencyManager {
    pub fn new() -> DependencyManager {
        DependencyManager {
            pub_subs: DHashMap::default()
        }
    }

    fn get_obj_deps(&self, obj: &RefID, results: &mut HashSet<RefID>) {
        if let Some(set) = self.pub_subs.get(obj) {
            for key in &(*set) {
                if results.insert(key.clone()) {
                    self.get_obj_deps(key, results);
                }
            }
        }
    }

    pub fn get_deps(&self, obj: &RefID) -> HashSet<RefID> {
        let mut results = HashSet::new();
        results.insert(*obj);  //We have to do some fancy things here to make sure we don't get cycles.
        self.get_obj_deps(obj, &mut results);
        results.remove(obj);
        results
    }

    pub fn get_all_deps(&self, ids: &Vec<RefID>) -> HashSet<RefID> 
    {
        let mut results = HashSet::new();
        for id in ids {
            results.insert(*id);
        }
        for id in ids {
            self.get_obj_deps(id, &mut results);
        }
        for id in ids {
            results.remove(id);
        }
        results
    }

    pub fn debug_state(&self, output: &mut String) {
        output.push_str(&format!("{:?} Dependencies:\n", self.pub_subs.len()));
        for chunk in self.pub_subs.chunks() {
            for (id, set) in chunk.iter() {
                output.push_str(&format!("{:?} -> {:?}\n", id, set));
            }
        }
    }

    pub fn register_sub(&self, publisher: &RefID, sub: RefID) {
        if sub != RefID::nil() {
            match self.pub_subs.get_mut(publisher) {
                Some(mut set) => {
                    set.insert(sub);
                }
                None => {
                    let mut set = HashSet::new();
                    set.insert(sub);
                    self.pub_subs.insert(publisher.clone(), set);
                }
            }
        }
    }

    pub fn delete_sub(&self, publisher: &RefID, sub: &RefID) {
        if let Some(mut set) = self.pub_subs.get_mut(publisher) {
            set.remove(sub);
        }
    }

    pub fn delete_obj(&self, publisher: &RefID) {
        self.pub_subs.remove(publisher);
        self.pub_subs.alter(|(_, set)| {
            set.remove(publisher);
        });
    }
}

