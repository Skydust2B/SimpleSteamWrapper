use slint::{ComponentHandle};
use crate::{HardRefresh};
use crate::gui::globals::global_init_trait::GlobalInitializer;
use crate::slint_utils::WeakUtils;

impl<T> GlobalInitializer<T> for HardRefresh<'_>
where
    T: ComponentHandle + 'static,
    for<'a> HardRefresh<'a>: slint::Global<'a, T>,
{
    type Ctx = Box<dyn Fn() + 'static>;

    fn init_global(component: &T, on_refresh: Self::Ctx) {
        let hard_refresh_globals = component.global::<HardRefresh>();

        hard_refresh_globals.on_force_refresh({
            let weak_window = component.as_weak();
            move || {
                weak_window.upgrade_and_run(|window| {
                    let hard_refresh_globals = window.global::<HardRefresh>();
                    hard_refresh_globals.set_refresh(false);
                    window.window().request_redraw();
                });
                on_refresh();
                let _ = weak_window.upgrade_in_event_loop(|window| {
                    let hard_refresh_globals = window.global::<HardRefresh>();
                    hard_refresh_globals.set_refresh(true);
                    window.window().request_redraw();
                });
            }
        });
    }
}

pub trait WindowForceRefresh {
    fn force_refresh(&self);
}

impl<T> WindowForceRefresh for T
where
    T: ComponentHandle + 'static,
    for<'a> HardRefresh<'a>: slint::Global<'a, T>,
{
    fn force_refresh(&self) {
        let hard_refresh_globals = self.global::<HardRefresh>();
        hard_refresh_globals.invoke_force_refresh();
    }
}
