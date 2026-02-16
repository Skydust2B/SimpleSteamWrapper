use std::cell::RefCell;
use std::rc::Rc;
use slint::{ModelRc, SharedString, VecModel};

#[derive(Clone)]
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
            VecModel::from(
            elements
                .iter()
                .map(|dc| SharedString::from(mapping_fn(dc)))
            .collect::<Vec<SharedString>>())).into()
    }

    pub fn get_from_idx(&self, index: i32) -> T {
        let elements = self.elements.borrow();
        elements[index as usize].clone()
    }
}
