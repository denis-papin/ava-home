use log::info;

#[derive(Debug, Clone)]
pub (crate) struct DeviceLock<T> {
    pub count_locks : u32,
    pub last_object_message : T,
}

impl <T> DeviceLock<T> {
    pub (crate) fn new(last_message: T) -> Self {
        Self {
            count_locks: 0,
            last_object_message: last_message,
        }
    }

    pub (crate) fn inc(&mut self) {
        self.count_locks += 1;
        info!("🔼 After up Locks:[{}]", self.count_locks);
    }
    pub (crate) fn dec(&mut self) {
        self.count_locks -= 1;
        info!("⏬ After down Locks:[{}]", self.count_locks);
    }

    pub (crate) fn replace(&mut self, o : T) {
        self.last_object_message = o;
    }

}
