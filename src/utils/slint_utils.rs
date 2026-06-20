use std::cell::RefCell;
use std::rc::Rc;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use crate::utils::find_index::FindIndex;

#[derive(Clone, Debug)]
pub struct ClonableModel<T> {
    elements: Rc<RefCell<Vec<T>>>
}

impl<T: Clone> ClonableModel<T> {
    pub fn new(elements: Vec<T>) -> Self {
        ClonableModel { elements: Rc::new(RefCell::new(elements)) }
    }

    pub fn set_model(&self, new_elements: Vec<T>) {
        *self.elements.borrow_mut() = new_elements;
    }

    pub fn to_model_rc<F>(&self, mapping_fn: F) -> ModelRc<SharedString>
    where F: Fn(&T) -> String {
        let elements = self.elements.borrow();
        Rc::new(
            elements
                .iter()
                .map(|dc| SharedString::from(mapping_fn(dc)))
            .collect::<VecModel<SharedString>>()).into()
    }

    pub fn get_from_idx(&self, index: i32) -> Option<T> {
        let elements = self.elements.borrow();
        elements.get(index as usize)
            .and_then(|e| Some(e.clone()))
    }
}

impl<T> FindIndex<T> for ClonableModel<T> {
    fn find_index<F>(&self, predicate: F) -> Option<i32>
    where
        F: Fn(&T) -> bool
    {
        self.elements.borrow()
            .iter()
            .position(predicate)
            .and_then(|idx| i32::try_from(idx).ok())
    }
}

pub trait WeakUtils<T> {
    fn upgrade_and_run<F>(&self, run: F)
    where F: FnOnce(T);
}

impl<T> WeakUtils<T> for Weak<T> where
    T: ComponentHandle {
    fn upgrade_and_run<F>(&self, run: F)
    where
        F: FnOnce(T)
    {
        if let Some(window) = self.upgrade() {
            run(window);
        }
    }
}
