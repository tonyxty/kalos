use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::OptimizationLevel;

pub struct JITExecutionEngine<'a, 'ctx> {
    module: &'a Module<'ctx>,
    engine: ExecutionEngine<'ctx>,
}

impl<'a, 'ctx> JITExecutionEngine<'a, 'ctx> {
    pub fn new(module: &'a Module<'ctx>) -> Self {
        Self {
            module,
            engine: module.create_jit_execution_engine(OptimizationLevel::Default).unwrap(),
        }
    }

    pub fn get_main(&self) -> JitFunction<'ctx, unsafe extern "C" fn() -> i64> {
        unsafe { self.engine.get_function("main") }.unwrap()
    }
}

impl JITExecutionEngine<'_, '_> {
    pub fn attach_runtime<'a, T>(&self, runtime: impl IntoIterator<Item=&'a (&'a T, usize)>)
        where T: 'a + ?Sized + AsRef<str>   // can't pretend I understand what I wrote
    {
        for (name, addr) in runtime {
            if let Some(func) = self.module.get_function(name.as_ref()) {
                self.engine.add_global_mapping(&func, *addr);
            }
        }
    }
}
