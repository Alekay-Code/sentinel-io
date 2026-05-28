use std::cell::OnceCell;

use super::runtime::runtime::Runtime;

pub struct Context {
    pub runtime: OnceCell<Runtime>,
}

thread_local! {
    pub static CONTEXT: Context = Context {
        runtime: OnceCell::new(),
    };
}

impl Context {
    pub fn init_runtime(runtime: Runtime) {
        CONTEXT.with(|ctx| {
            if ctx.runtime.set(runtime).is_err() {
                panic!("Another runtime was already created")
            }
        });
    }

    // Panic if runtime doesn't exists
    pub fn with_runtime<T>(f: impl FnOnce(&Runtime) -> T) -> T {
        CONTEXT.with(|ctx| {
            let rt = ctx.runtime.get().expect("Runtime has not been created");
            f(rt)
        })
    }
}
