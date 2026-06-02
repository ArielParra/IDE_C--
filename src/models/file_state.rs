use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub type FileState = Rc<RefCell<Option<PathBuf>>>;

pub fn new_file_state() -> FileState {
    Rc::new(RefCell::new(None))
}
