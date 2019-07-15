use crate::{RefID, DepStore};
use ccl::dhashmap::DHashMap;
use std::collections::HashSet;

pub struct DependencyManager {
    pub_subs: DHashMap<RefID, HashSet<RefID>>
}

impl DependencyManager {
    pub fn new() -> DependencyManager {
        DependencyManager {
            pub_subs: DHashMap::default()
        }
    }

    pub fn get_deps(&self, obj: &RefID) -> HashSet<RefID> {
        let mut results = HashSet::new();
        if let Some(set) = self.pub_subs.get(obj) {
            results = set.clone()
        }
        results
    }

    pub fn get_all_deps<'a>(&self, ids: impl Iterator<Item=&'a RefID>) -> HashSet<RefID> 
    {
        let mut results = HashSet::new();
        for id in ids {
            if let Some(set) = self.pub_subs.get(id) {
                results.extend(set.clone());
            }
        }
        results
    }
}

impl DepStore for DependencyManager {
    fn register_sub(&self, publisher: &RefID, sub: RefID) {
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

    fn delete_sub(&self, publisher: &RefID, sub: &RefID) {
        if let Some(mut set) = self.pub_subs.get_mut(publisher) {
            set.remove(sub);
        }
    }

    fn delete_obj(&self, publisher: &RefID) {
        self.pub_subs.remove(publisher);
        self.pub_subs.alter(|(_, set)| {
            set.remove(publisher);
        });
    }
}
