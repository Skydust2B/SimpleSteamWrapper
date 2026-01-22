use slint::{ComponentHandle, Weak};

pub trait InvokeFromEventLoop {
    type Target;

    fn invoke<F>(&self, f: F)
    where
        F: FnOnce(Self::Target) + Send + 'static;
}

impl<T> InvokeFromEventLoop for Weak<T>
where
    T: ComponentHandle + 'static
{
    type Target = T;
    fn invoke<F>(&self, f: F)
    where
        F: FnOnce(T) + Send + 'static,
    {
        let weak = self.clone();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = weak.upgrade() {
                f(window);
            }
        });
    }
}
