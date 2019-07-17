#![allow(dead_code)]
extern crate ccl;
#[macro_use] extern crate lazy_static;
extern crate tokio;
extern crate futures;
extern crate tokio_threadpool;
extern crate crossbeam_channel;
extern crate data_model;
#[cfg(test)] #[macro_use]
extern crate query_interface;
#[cfg(test)] #[macro_use]
extern crate serde_json;

#[cfg(test)]
mod tests;

mod operation_manager;
mod scheduler;
mod app_state;
mod entity_ops;

pub use data_model::*;
pub use entity_ops::*;

pub use std::path::PathBuf;
pub use std::collections::{HashSet, HashMap, VecDeque};
pub use app_state::{init_file, begin_undo_event, end_undo_event, undo_latest, redo_latest, suspend_event, resume_event, cancel_event, take_undo_snapshot, delete_obj};
