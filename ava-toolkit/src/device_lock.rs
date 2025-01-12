use log::*;

#[derive(Debug, Clone)]
pub struct DeviceLock<T> {
    pub count_locks : u32,
    pub last_object_message : T,
}

impl <T> DeviceLock<T> {
    pub fn new(last_message: T) -> Self {
        Self {
            count_locks: 0,
            last_object_message: last_message,
        }
    }

    pub fn inc(&mut self) {
        self.count_locks += 1;
        info!("🔼 After up Locks:[{}]", self.count_locks);
    }
    pub fn dec(&mut self) {
        self.count_locks -= 1;
        info!("⏬ After down Locks:[{}]", self.count_locks);
    }

    pub fn replace(&mut self, o : T) {
        self.last_object_message = o;
    }

}
