use ccl::dhashmap::DHashMap;
use crate::*;
use super::undo::{UndoEvent, Change};
use super::{DBError, DataObject};

pub struct FileDatabase {
    db: DHashMap<RefID, DataObject>,
}

impl FileDatabase {
    pub fn new() -> FileDatabase {
        FileDatabase{ 
            db: DHashMap::default(),
        }
    }

    pub fn add(&self, obj: DataObject) -> Result<(), DBError> {
        let key = obj.get_id();
        if !self.db.contains_key(key) {
            self.db.insert(key.clone(), obj);
            Ok(())
        }
        else {
            Err(DBError::Overwrite)
        }
    }

    pub fn get(&self, key: &RefID, callback: &mut FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        if *key == RefID::nil() {
            return Err(DBError::NotFound);
        }
        match self.db.get(key) {
            Some(obj) => callback(&(*obj)),
            None => Err(DBError::NotFound)
        }
    }

    pub fn remove(&self, key: &RefID) -> Result<DataObject, DBError> {
        match self.db.remove(key) {
            Some(val) => Ok(val.1),
            None => Err(DBError::NotFound),
        }
    }

    pub fn get_mut(&self, key: &RefID, callback: &mut FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        if *key == RefID::nil() {
            return Err(DBError::NotFound);
        }
        match self.db.get_mut(key) {
            Some(mut obj) => callback(&mut (*obj)),
            None => Err(DBError::NotFound)
        }
    }

    pub fn undo(&self, event: UndoEvent) -> Result<UndoEvent, DBError> {
        let mut redo = event.clone();
        redo.changes.clear();
        for change in event.changes.iter().rev() {
            match change {
                Change::Add{key} => {
                    match self.db.remove(&key) {
                        Some(val) => redo.changes.push(Change::Delete{obj: val.1}),
                        None => return Err(DBError::NotFound),
                    }
                }
                Change::Modify{obj} => {
                    match self.db.remove(obj.get_id()) {
                        Some(val) => {
                            redo.changes.push(Change::Modify{obj: val.1});
                        }
                        None => return Err(DBError::NotFound),
                    }
                    self.db.insert(obj.get_id().clone(), obj.clone());
                }
                Change::Delete{obj} => {
                    redo.changes.push(Change::Add{key: obj.get_id().clone()});
                    self.db.insert(obj.get_id().clone(), obj.clone());
                }
            }
        }
        Ok(redo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn test_read() {
        let db = FileDatabase::new();
        let obj = Box::new(TestObj::new("some data"));
        let id = obj.get_id().clone();
        db.add(obj).unwrap();
        db.get(&id, &mut|read:&DataObject| {
            let data = read.query_ref::<Store>().unwrap().get_store_data();
            assert_eq!(String::from("some data"), data);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_write() {
        let db = FileDatabase::new();
        let obj = Box::new(TestObj::new("some data"));
        let id = obj.get_id().clone();
        db.add(obj).unwrap();
        {
            db.get_mut(&id, &mut|to_modify:&mut DataObject| {
                to_modify.query_mut::<Store>().unwrap().set_store_data(String::from("new data"));
                Ok(())
            }).unwrap();
        }
        db.get(&id, &mut|read:&DataObject| {
            let data = read.query_ref::<Store>().unwrap().get_store_data();
            assert_eq!(String::from("new data"), data);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_delete() {
        let db = FileDatabase::new();
        let obj = Box::new(TestObj::new("some data"));
        let id = obj.get_id().clone();
        db.add(obj).unwrap();
        let removed = db.remove(&id).unwrap();
        let data = removed.query_ref::<Store>().unwrap().get_store_data();
        assert_eq!(String::from("some data"), data);
        assert!(db.get(&id, &mut|_| {Ok(())}).is_err());
    }

    #[test]
    fn test_duplicate() {
        let db = FileDatabase::new();
        let obj = Box::new(TestObj::new("some data"));
        db.add(obj.clone()).unwrap();
        assert!(db.add(obj).is_err());
    }
}
