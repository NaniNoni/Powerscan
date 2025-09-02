use crate::{SANE_Action, SANE_Handle, sane_close};

// https://sane-project.gitlab.io/standard/api.html#scanner-handle-type
pub struct Handle {
    pub raw: SANE_Handle,
}

impl Handle {
    pub fn control_option<T>(&self, option: i32, action: SANE_Action, value: &mut T) {}
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { sane_close(self.raw) }
    }
}
