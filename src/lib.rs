use std::marker::PhantomData;

mod platform;

pub struct Window {
    
}

pub struct EventLoop {
    _no_send_sync: PhantomData<*mut ()>,
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
