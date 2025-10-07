use std::{cell::RefCell, fmt::Debug, hash::Hash, ops::Deref, rc::Rc};

#[derive(Clone, PartialEq)]
pub struct Handle<T: ?Sized>(Rc<RefCell<T>>);

impl<T> Debug for Handle<T> where T: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Handle").field(&*self.0).finish()
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = self.0.as_ref();
        core::ptr::hash(ptr, state);
    }
}

impl<T: ?Sized> Deref for Handle<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Handle<T> {
    pub fn new(x: T) -> Self {
        Self(Rc::new(RefCell::new(x)))
    }
}
