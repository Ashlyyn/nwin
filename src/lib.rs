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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
