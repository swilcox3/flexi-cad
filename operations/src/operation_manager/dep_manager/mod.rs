use crate::prelude::*;
use ccl::dhashmap::DHashMap;

pub struct DependencyManager {
    pub_subs: DHashMap<RefRecord, HashSet<RefRecord>>
}

impl DependencyManager {
    pub fn new() -> DependencyManager {
        DependencyManager {
            pub_subs: DHashMap::default()
        }
    }

    fn breadth_first_search(&self, obj: Reference) -> Vec<HashSet<RefSource>> {
        let mut processing = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        visited.insert(obj.clone());
        processing.push_back(obj.clone());
        while processing.len() > 0 {
            if let Some(current) = processing.pop_front() {
                if let Some(sub_set) = self.pub_subs.get(&current) {
                    let mut cur_level = HashSet::new();
                    for sub in &(*sub_set) {
                        if let None = visited.get(&sub.id) {
                            visited.insert(sub.id.clone());
                            cur_level.insert(sub.clone());
                            processing.push_back(sub.id.clone());
                        }
                    }
                    if cur_level.len() > 0 {
                        result.push(cur_level);
                    }
                }
            }
        }
        result
    }

    pub fn get_all_deps<T>(&self, objs: T) -> Vec<Reference> 
        where T: IntoIterator<Item = RefID> 
    {
        let mut results = Vec::new();
        let mut levels: Vec<HashSet<RefID>> = Vec::new();
        for obj in objs.into_iter() {
            let mut i = 0;
            for level in self.get_deps(&obj) {
                if let Some(exists) = levels.get_mut(i) {
                    exists.extend(level);
                }
                else {
                    levels.push(level);
                }
                i += 1;
            }
        }
        for level in levels {
            for obj in level {
                results.push(obj);
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

    pub fn register_sub(&self, publisher: &RefRecord, sub: RefRecord) {
        if sub != RefID::nil() && *publisher != sub {
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

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! set {
        ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
            {
                let mut temp_set = HashSet::new();  // Create a mutable HashSet
                $(
                    temp_set.insert($x); // Insert each item matched into the HashSet
                )*
                temp_set // Return the populated HashSet
            }
        };
    }

    #[test]
    fn test_get_deps() {
        let deps = DependencyManager::new();
        let a = RefID::new_v4();
        let b = RefID::new_v4();
        let c = RefID::new_v4();
        let d = RefID::new_v4();
        let e = RefID::new_v4();
        let f = RefID::new_v4();

        deps.register_sub(&a, b.clone());
        deps.register_sub(&a, c.clone());
        deps.register_sub(&a, d.clone());
        deps.register_sub(&b, a.clone());
        deps.register_sub(&b, c.clone());
        deps.register_sub(&c, a.clone());
        deps.register_sub(&c, d.clone());
        deps.register_sub(&d, a.clone());
        deps.register_sub(&e, b.clone());
        deps.register_sub(&e, c.clone());
        deps.register_sub(&e, d.clone());
        deps.register_sub(&e, f.clone());

        assert_eq!(deps.get_deps(&a), vec![set![b.clone(), c.clone(), d.clone()]]);
        assert_eq!(deps.get_deps(&b), vec![set![a.clone(), c.clone()], set![d.clone()]]);
        assert_eq!(deps.get_deps(&c), vec![set![a.clone(), d.clone()], set![b.clone()]]);
        assert_eq!(deps.get_deps(&d), vec![set![a.clone()], set![b.clone(), c.clone()]]);
        assert_eq!(deps.get_deps(&e), vec![set![b.clone(), c.clone(), d.clone(), f.clone()], set![a.clone()]]);
        assert_eq!(deps.get_deps(&f), vec![]);
    }

    fn set_exists_within_range(mut set: HashSet<RefID>, base: &Vec<RefID>, index: usize, size: usize) -> bool {
        for i in index..index + size {
            set.remove(&base[i]);
        }
        set.len() == 0
    }

    fn deps_equals(input: Vec<RefID>, answers: Vec<HashSet<RefID>>) -> bool {
        let mut cur_index = 0;
        for set in answers {
            let size = set.len();
            if !set_exists_within_range(set, &input, cur_index, size) {
                return false;
            }
            cur_index += size;
        }
        true
    }

    #[test]
    fn test_get_all_deps() {
        let deps = DependencyManager::new();
        let a = RefID::new_v4();
        let b = RefID::new_v4();
        let c = RefID::new_v4();
        let d = RefID::new_v4();
        let e = RefID::new_v4();
        let f = RefID::new_v4();

        deps.register_sub(&a, b.clone());
        deps.register_sub(&a, c.clone());
        deps.register_sub(&a, d.clone());
        deps.register_sub(&b, a.clone());
        deps.register_sub(&b, c.clone());
        deps.register_sub(&c, a.clone());
        deps.register_sub(&c, d.clone());
        deps.register_sub(&d, a.clone());
        deps.register_sub(&e, b.clone());
        deps.register_sub(&e, c.clone());
        deps.register_sub(&e, d.clone());
        deps.register_sub(&e, f.clone());

        let input = deps.get_all_deps(vec![a.clone(), e.clone()]);
        let answers = vec![set![b.clone(), c.clone(), d.clone(), f.clone()]];
        assert!(deps_equals(input, answers));

        let input = deps.get_all_deps(vec![d.clone(), f.clone()]);
        let answers = vec![set![a.clone()], set![b.clone(), c.clone()]];
        assert!(deps_equals(input, answers));

        let input = deps.get_all_deps(vec![f.clone()]);
        assert_eq!(input.len(), 0);
    }
}

