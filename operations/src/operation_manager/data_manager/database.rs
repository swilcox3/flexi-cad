use ccl::dhashmap::{DHashMap, TryGetError};
use crate::prelude::*;
use super::undo::{UndoEvent, Change};
use std::io::{Read, Write};
use std::time::Duration;

fn print_stacktrace() {
    backtrace::trace(|frame| {
        backtrace::resolve_frame(frame, |symbol| {
            if let Some(name) = symbol.name() {
                println!("{:?}", name);
            }
        });

        true // keep going to the next frame
    });
}

fn run_timeout(mut callback: impl FnMut() -> Result<(), DBError>) -> Result<(), DBError> {
    let now = std::time::SystemTime::now();
    let timeout = 5000;
    let wait = Duration::from_millis(10);
    loop {
        match callback() {
            Ok(()) => {
                return Ok(());
            }
            Err(DBError::TimedOut) => (),
            Err(e) => {
                return Err(e);
            }
        }
        if let Ok(elapsed) = now.elapsed() {
            if elapsed.as_millis() > timeout {
                return Err(DBError::TimedOut);
            }
        }
        std::thread::sleep(wait);
    }
}

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

    pub fn get(&self, key: &RefID, mut callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        if *key == RefID::nil() {
            return Err(DBError::NotFound);
        }
        let try_get = || {
            match self.db.try_get(key) {
                Ok(obj) => callback(&(*obj)),
                Err(TryGetError::WouldBlock) => Err(DBError::TimedOut),
                _ => Err(DBError::NotFound),
            }
        };
        run_timeout(try_get)
    }

    pub fn iterate_all(&self, mut callback: impl FnMut(&DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        for chunk in self.db.chunks() {
            for (_, val) in chunk.iter() {
                callback(val)?;
            }
        }
        Ok(())
    }

    pub fn remove(&self, key: &RefID) -> Result<DataObject, DBError> {
        match self.db.remove(key) {
            Some(val) => Ok(val.1),
            None => Err(DBError::NotFound),
        }
    }

    pub fn get_mut(&self, key: &RefID, mut callback: impl FnMut(&mut DataObject) -> Result<(), DBError>) -> Result<(), DBError> {
        if *key == RefID::nil() {
            return Err(DBError::NotFound);
        }
        let try_get = || {
            match self.db.try_get_mut(key) {
                Ok(mut obj) => callback(&mut (*obj)),
                Err(TryGetError::WouldBlock) => Err(DBError::TimedOut),
                _ => Err(DBError::NotFound),
            }
        };
        run_timeout(try_get)
    }

    pub fn duplicate(&self, key: &RefID) -> Result<DataObject, DBError> {
        if *key == RefID::nil() {
            return Err(DBError::NotFound);
        }
        match self.db.get(key) {
            Some(obj) => {
                let mut copy = obj.clone();
                let id = RefID::new_v4();
                copy.set_id(id);
                Ok(copy)
            }
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

    pub fn save(&self, path: &PathBuf) -> Result<(), DBError> {
        let mut file = std::fs::File::create(path).map_err(error_other)?;
        let mut vals = Vec::new();
        for chunk in self.db.chunks() {
            for (_, val) in chunk.iter() {
                vals.push(val.clone());
            }
        }
        let buf = bincode::serialize(&vals).map_err(error_other)?;
        file.write_all(&buf).map_err(error_other)?;
        Ok(())
    }

    pub fn open(&self, path: &PathBuf) -> Result<(), DBError> {
        let mut file = std::fs::File::open(path).map_err(error_other)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(error_other)?;
        let objects: Vec<DataObject> = bincode::deserialize(&buf).map_err(error_other)?;
        for obj in objects {
            self.add(obj)?;
        }
        Ok(())
    }

    pub fn debug_state(&self, output: &mut String) {
        output.push_str(&format!("{:?} Objects:\n", self.db.len()));
        for chunk in self.db.chunks() {
            for (_, obj) in chunk.iter() {
                output.push_str(&format!("{:?}\n", obj));
            }
        }
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
        assert!(db.get(&id, |_| {Ok(())}).is_err());
    }

    #[test]
    fn test_duplicate() {
        let db = FileDatabase::new();
        let obj = Box::new(TestObj::new("some data"));
        db.add(obj.clone()).unwrap();
        assert!(db.add(obj).is_err());
    }

    #[test]
    fn test_filing() {
        let path = PathBuf::from("./test_file.flx");
        let obj_1 = Box::new(TestObj::new("first"));
        let obj_2 = Box::new(TestObj::new("second"));
        let obj_3 = Box::new(TestObj::new("third"));
        let id_1 = obj_1.get_id().clone();
        let id_2 = obj_2.get_id().clone();
        let id_3 = obj_3.get_id().clone();
        {
            let db = FileDatabase::new();
            db.add(obj_1).unwrap();
            db.add(obj_2).unwrap();
            db.add(obj_3).unwrap();
            db.save(&path).unwrap();
        }
        let db = FileDatabase::new();
        db.open(&path).unwrap();
        db.get(&id_1, &mut|obj:&DataObject| {
            let data = obj.query_ref::<Store>().unwrap().get_store_data();
            assert_eq!(String::from("first"), data);
            Ok(())
        }).unwrap();
        db.get(&id_2, &mut|obj:&DataObject| {
            let data = obj.query_ref::<Store>().unwrap().get_store_data();
            assert_eq!(String::from("second"), data);
            Ok(())
        }).unwrap();
        db.get(&id_3, &mut|obj:&DataObject| {
            let data = obj.query_ref::<Store>().unwrap().get_store_data();
            assert_eq!(String::from("third"), data);
            Ok(())
        }).unwrap();
        std::fs::remove_file(path).unwrap();
    }


}
