use slint::ComponentHandle;

pub trait GlobalInitializer<T>
where
    T: ComponentHandle
{
    type Ctx;
    fn init_global(component: &T, ctx: Self::Ctx) where Self: Sized;
}

pub trait ComponentInitExt {
    fn init_global<G>(&self, ctx: G::Ctx)
    where
        G: GlobalInitializer<Self>,
        Self: Sized,
        Self: ComponentHandle;
}

impl<T> ComponentInitExt for T
where
    T: ComponentHandle,
{
    fn init_global<G>(&self, ctx: G::Ctx)
    where
        G: GlobalInitializer<T>,
    {
        G::init_global(self, ctx);
    }
}

