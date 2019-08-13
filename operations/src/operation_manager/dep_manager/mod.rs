use crate::*;
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

