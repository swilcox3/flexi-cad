use crate::prelude::*;
use ccl::dhashmap::DHashMap;

pub struct DependencyManager {
    pub_subs: DHashMap<GeometryId, HashSet<GeometryId>>
}

impl DependencyManager {
    pub fn new() -> DependencyManager {
        DependencyManager {
            pub_subs: DHashMap::default()
        }
    }

    fn breadth_first_search(&self, obj: GeometryId) -> Vec<HashSet<Reference>> {
        let mut processing = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        visited.insert(obj.clone());
        processing.push_back(obj);
        while processing.len() > 0 {
            if let Some(current) = processing.pop_front() {
                if let Some(sub_set) = self.pub_subs.get(&current) {
                    let mut cur_level = HashSet::new();
                    for sub in &(*sub_set) {
                        if let None = visited.get(&sub) {
                            visited.insert(sub.clone());
                            cur_level.insert(Reference{ owner: sub.clone(), other: current.clone() } );
                            processing.push_back(sub.clone());
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
        where T: IntoIterator<Item = GeometryId> 
    {
        let mut levels: Vec<HashSet<Reference>> = Vec::new();
        for obj in objs.into_iter() {
            let mut i = 0;
            for level in self.breadth_first_search(obj) {
                if let Some(exists) = levels.get_mut(i) {
                    exists.extend(level);
                }
                else {
                    levels.push(level);
                }
                i += 1;
            }
        }
        let mut results = Vec::new();
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

    pub fn register_sub(&self, publisher: &GeometryId, sub: GeometryId) {
        if sub.obj != RefID::nil() && *publisher != sub {
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

    pub fn delete_obj(&self, publisher: &GeometryId) {
        self.pub_subs.remove(publisher);
        self.pub_subs.alter(|(_, set)| {
            set.remove(publisher);
        });
    }

    pub fn delete_objs(&self, objs: HashSet<RefID>) {
        self.pub_subs.retain(|key, _| !objs.contains(&key.obj));
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
        Reference::new(owner.clone(), other.clone())
    }

    fn set_exists_within_range(mut set: HashSet<Reference>, base: &Vec<Reference>, index: usize, size: usize) -> bool {
        for i in index..index + size {
            set.remove(&base[i]);
        }
        set.len() == 0
    }

    fn deps_equals(input: Vec<Reference>, answers: Vec<HashSet<Reference>>) -> bool {
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
        //This simulates three walls with windows in each one.
        let a = RefID::new_v4();
        let a_1 = GeometryId{ obj: a.clone(), index: 0 };
        let a_2 = GeometryId{ obj: a.clone(), index: 1 };
        let a_3 = GeometryId{ obj: a.clone(), index: 2 };
        let b = RefID::new_v4();
        let b_1 = GeometryId{ obj: b.clone(), index: 0 };
        let b_2 = GeometryId{ obj: b.clone(), index: 1 };
        let b_3 = GeometryId{ obj: b.clone(), index: 2 };
        let c = RefID::new_v4();
        let c_1 = GeometryId{ obj: c.clone(), index: 0 };
        let c_2 = GeometryId{ obj: c.clone(), index: 1 };
        let c_3 = GeometryId{ obj: c.clone(), index: 2 };
        let d = RefID::new_v4();
        let d_1 = GeometryId{ obj: d.clone(), index: 0 };
        let d_2 = GeometryId{ obj: d.clone(), index: 1 };
        let e = RefID::new_v4();
        let e_1 = GeometryId{ obj: e.clone(), index: 0 };
        let e_2 = GeometryId{ obj: e.clone(), index: 1 };
        let f = RefID::new_v4();
        let f_1 = GeometryId{ obj: f.clone(), index: 0 };
        let f_2 = GeometryId{ obj: f.clone(), index: 1 };

        deps.register_sub(&a_1, b_1.clone());
        deps.register_sub(&a_1, d_1.clone());
        deps.register_sub(&a_2, c_2.clone());
        deps.register_sub(&a_2, d_2.clone());
        deps.register_sub(&b_1, a_1.clone());
        deps.register_sub(&b_1, e_1.clone());
        deps.register_sub(&b_2, e_2.clone());
        deps.register_sub(&c_1, f_1.clone());
        deps.register_sub(&c_2, a_2.clone());
        deps.register_sub(&c_2, f_2.clone());
        deps.register_sub(&d_1, a_3.clone());
        deps.register_sub(&e_1, b_3.clone());
        deps.register_sub(&f_1, c_3.clone());

        let results = deps.get_all_deps(vec![a_1.clone()]);
        let answer =
            vec![
                set![
                    get_ref(&b_1, &a_1),
                    get_ref(&d_1, &a_1)
                ],
                set![
                    get_ref(&e_1, &b_1),
                    get_ref(&a_3, &d_1)
                ],
                set![get_ref(&b_3, &e_1)]
            ];
        assert!(deps_equals(results, answer));

        let results = deps.get_all_deps(vec![b_2.clone()]);
        let answer = 
            vec![
                set![get_ref(&e_2, &b_2)],
            ];
        assert!(deps_equals(results, answer));

        let results = deps.get_all_deps(vec![a_1.clone(), a_2.clone()]);
        let answer = 
            vec![
                set![
                    get_ref(&b_1, &a_1),
                    get_ref(&d_1, &a_1),
                    get_ref(&c_2, &a_2),
                    get_ref(&d_2, &a_2)
                ],
                set![
                    get_ref(&e_1, &b_1),
                    get_ref(&a_3, &d_1),
                    get_ref(&f_2, &c_2)
                ],
                set![
                    get_ref(&b_3, &e_1)
                ]
            ];
        assert!(deps_equals(results, answer));
    }

}

