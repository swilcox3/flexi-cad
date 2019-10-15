use crate::prelude::*;
use ccl::dhashmap::DHashMap;
use indexmap::IndexSet;

pub struct DependencyManager {
    pub_subs: DHashMap<GeometryId, HashSet<GeometryId>>,
}

impl DependencyManager {
    pub fn new() -> DependencyManager {
        DependencyManager {
            pub_subs: DHashMap::default(),
        }
    }

    fn breadth_first_search(&self, id: &GeometryId) -> IndexSet<Reference> {
        let mut processing = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        let mut result = IndexSet::new();
        visited.insert(id.clone());
        processing.push_back(id.clone());
        while processing.len() > 0 {
            if let Some(current) = processing.pop_front() {
                if let Some(sub_set) = self.pub_subs.get(&current) {
                    for sub in &(*sub_set) {
                        if let None = visited.get(sub) {
                            visited.insert(sub.clone());
                            result.insert(Reference {
                                owner: sub.clone(),
                                other: current.clone(),
                            });
                            processing.push_back(sub.clone());
                        }
                    }
                }
            }
        }
        result
    }

    pub fn get_all_deps<T>(&self, ids: T) -> IndexSet<Reference>
    where
        T: IntoIterator<Item = GeometryId>,
    {
        /*let mut debug = String::new();
        self.debug_state(&mut debug);
        println!("{}", debug);*/
        let mut results: IndexSet<Reference> = IndexSet::new();
        for id in ids.into_iter() {
            results.extend(self.breadth_first_search(&id));
        }
        results
    }

    pub fn debug_state(&self, output: &mut String) {
        output.push_str(&format!("{:?} Dependencies:\n", self.pub_subs.len()));
        for chunk in self.pub_subs.chunks() {
            for (refer, set) in chunk.iter() {
                output.push_str(&format!("{:?} -> {:?}\n", refer, set));
            }
        }
    }

    pub fn register_sub(&self, publisher: &GeometryId, sub: GeometryId) {
        if sub.id != RefID::nil() && *publisher != sub {
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

    pub fn delete_sub(&self, publisher: &GeometryId, sub: &GeometryId) {
        if let Some(mut set) = self.pub_subs.get_mut(publisher) {
            set.remove(sub);
        }
    }

    pub fn delete_id(&self, publisher: &GeometryId) {
        self.pub_subs.remove(publisher);
        self.pub_subs.alter(|(_, set)| {
            set.remove(publisher);
        });
    }

    pub fn delete_ids(&self, ids: HashSet<RefID>) {
        self.pub_subs.retain(|key, _| !ids.contains(&key.id));
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

    fn get_ref(owner: &GeometryId, other: &GeometryId) -> Reference {
        Reference {
            owner: owner.clone(),
            other: other.clone(),
        }
    }

    fn set_exists_within_range(mut set: HashSet<Reference>, base: &IndexSet<Reference>, index: usize, size: usize) -> bool {
        for i in index..index + size {
            let entry = base.get_index(i).unwrap();
            set.remove(&entry);
        }
        set.len() == 0
    }

    fn deps_equals(input: IndexSet<Reference>, answers: Vec<HashSet<Reference>>) -> bool {
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
        let a_0 = GeometryId::new(a.clone(), 0);
        let a_1 = GeometryId::new(a.clone(), 1);
        let a_2 = GeometryId::new(a.clone(), 2);
        let b = RefID::new_v4();
        let b_0 = GeometryId::new(b.clone(), 0);
        let b_1 = GeometryId::new(b.clone(), 1);
        let b_2 = GeometryId::new(b.clone(), 2);
        let c = RefID::new_v4();
        let c_0 = GeometryId::new(c.clone(), 0);
        let c_1 = GeometryId::new(c.clone(), 1);
        let c_2 = GeometryId::new(c.clone(), 2);

        deps.register_sub(&a_0, b_0.clone());
        deps.register_sub(&a_1, c_2.clone());
        deps.register_sub(&a_1, a_2.clone());
        deps.register_sub(&b_1, c_0.clone());
        deps.register_sub(&b_2, a_1.clone());
        deps.register_sub(&c_0, b_1.clone());
        deps.register_sub(&c_1, c_0.clone());
        deps.register_sub(&c_2, c_1.clone());

        let results = deps.get_all_deps(vec![a_0.clone()]);
        let answer = vec![set![get_ref(&b_0, &a_0)]];
        assert!(deps_equals(results, answer));

        let results = deps.get_all_deps(vec![b_2.clone()]);
        let answer = vec![
            set![get_ref(&a_1, &b_2)],
            set![get_ref(&a_2, &a_1), get_ref(&c_2, &a_1)],
            set![get_ref(&c_1, &c_2)],
            set![get_ref(&c_0, &c_1)],
            set![get_ref(&b_1, &c_0)],
        ];
        assert!(deps_equals(results, answer));
    }
}
