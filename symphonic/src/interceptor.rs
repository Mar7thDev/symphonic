use ilhook::x64::*;

pub struct Interceptor {
    active_hooks: Vec<HookPoint>,
}

pub struct AttachContext {
    registers: *mut Registers,
}

pub struct ReplaceContext {
    registers: *mut Registers,
    pub original_fn: usize,
}

pub type AttachCallback = fn(ctx: &mut AttachContext);
pub type ReplaceCallback = fn(ctx: &mut ReplaceContext) -> usize;

impl Interceptor {
    pub fn new() -> Self {
        Self {
            active_hooks: Vec::new(),
        }
    }

    pub fn attach(&mut self, address: usize, callback: AttachCallback) {
        let hooker = Hooker::new(
            address,
            HookType::JmpBack(attach_callback),
            CallbackOption::None,
            callback as usize,
            HookFlags::empty(),
        );

        unsafe {
            if let Ok(hook_point) = hooker.hook() {
                self.active_hooks.push(hook_point);
            } else {
                eprintln!("failed to attach to 0x{address:X}");
            }
        }
    }

    pub fn replace(&mut self, address: usize, callback: ReplaceCallback) {
        let hooker = Hooker::new(
            address,
            HookType::Retn(replace_callback),
            CallbackOption::None,
            callback as usize,
            HookFlags::empty(),
        );

        unsafe {
            if let Ok(hook_point) = hooker.hook() {
                self.active_hooks.push(hook_point);
            } else {
                eprintln!("failed to attach to 0x{address:X}");
            }
        }
    }

    pub fn leak(self) {
        self.active_hooks.leak();
    }
}

impl AttachContext {
    pub fn registers(&mut self) -> &mut Registers {
        unsafe { &mut *self.registers }
    }
}

impl ReplaceContext {
    pub fn registers(&mut self) -> &mut Registers {
        unsafe { &mut *self.registers }
    }
}

unsafe extern "win64" fn attach_callback(reg: *mut Registers, actual_callback: usize) {
    let callback = unsafe { std::mem::transmute::<usize, AttachCallback>(actual_callback) };
    callback(&mut AttachContext { registers: reg });
}

unsafe extern "win64" fn replace_callback(
    reg: *mut Registers,
    original_fn: usize,
    actual_callback: usize,
) -> usize {
    let callback = unsafe { std::mem::transmute::<usize, ReplaceCallback>(actual_callback) };
    callback(&mut ReplaceContext {
        registers: reg,
        original_fn,
    })
}
