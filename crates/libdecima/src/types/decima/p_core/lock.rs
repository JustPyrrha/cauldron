use windows::Win32::System::Threading::SRWLOCK;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct SharedLock {
    pub lock: SRWLOCK,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct SharedLockProtected<T> {
    pub lock: SRWLOCK,
    pub data: T,
}
