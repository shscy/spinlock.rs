extern crate test;

use super::*;

use std::{sync::Arc, thread};

 #[test]
 fn read() {
     let spin = SpinLock::new(0u32);
     let x = spin.read().unwrap();
     assert_eq!(*x, 0u32);
 }

 #[test]
 fn write() {
     let spin = SpinLock::new(0u32);
     {
         let mut x = spin.write().unwrap();
         assert_eq!(*x, 0u32);
         *x = 42u32;
     }
     let x = spin.read().unwrap();
     assert_eq!(*x, 42u32);
 }

 #[test]
 fn try_read_success() {
     let spin = SpinLock::new(255);
     let result = spin.try_read();
     assert!(result.is_ok());
     let x = result.unwrap();
     assert_eq!(*x, 255);
 }

 #[test]
 fn try_write_success() {
     let spin = SpinLock::new(255);
     {
         let result = spin.try_write();
         assert!(result.is_ok());
         let mut x = result.unwrap();
         *x = 42;
     }
     let x = spin.read().unwrap();
     assert_eq!(*x, 42);
 }

 #[test]
 fn try_write_block() {
     let spin = SpinLock::new(255);
     let x = spin.write().unwrap();
     {
         let result = spin.try_write();
         assert!(match result { Err(TryLockError::WouldBlock) => true, _ => false });
     } 
     let _ = test::black_box(x);
 }

 #[test]
 fn try_read_block() {
     let spin = SpinLock::new(255);
     let x = spin.write().unwrap();
     {
         let result = spin.try_read();
         assert!(match result { Err(TryLockError::WouldBlock) => true, _ => false });
     }
     let _ = test::black_box(x);
 }

 #[test]
 #[should_panic]
 fn indefinite_read() {
     let spin = SpinLock::new(255);
     let x = spin.write().unwrap();
     {
         let _ = spin.read().unwrap();
     }
     let _ = test::black_box(x);
 }


struct PoisonnedGuard<'a, T : 'a>(&'a SpinLock<T>);


impl<'a, T> Drop for PoisonnedGuard<'a, T> {
    fn drop(&mut self) {
        println!("\n\ndrop grard sucess ***********");
        assert!(thread::panicking()); 
        assert!(self.0.is_poisonned());
    }
}

#[test]
fn poison() {
    // 1. 线程B 获取锁，然后强行panic 
    // 2. 主线程开始获取锁失败， 这个时候
    let spin = Arc::new(SpinLock::new(255));
    let s_clone2 = spin.clone();
    let _ = thread::spawn(move || { 
         let m  = s_clone2.write().unwrap();
         // 这里拿到锁之后强行panic
         panic!("panic by manual");
    }).join();
    let x = spin.write();
    assert!(spin.is_poisonned());
    match &x {
        Err(e) => println!("{:?}", e),
        _ => {panic!("should be poison")},
    }
    let _ = &x;
}

#[test]
fn concurrent_readers() {
    let spin = Arc::new(SpinLock::new(255));
    let mut workers : Vec<thread::JoinHandle<()>> = Vec::with_capacity(10); 
    for _ in (0..9) {
        let spin_clone = spin.clone();
        workers.push(thread::spawn(move || { let data = spin_clone.read().unwrap(); assert_eq!(*data, 255);}) );
    }
    for w in workers {
        w.join();
    }
}

 #[test]
 fn concurrent_writers() {
     let spin = Arc::new(SpinLock::new(255));
     let mut workers : Vec<thread::JoinHandle<()>> = Vec::with_capacity(10); 
     for i in (0u32..9) {
         let spin_clone = spin.clone();
         workers.push(thread::spawn(move || { let mut data = spin_clone.write().unwrap(); *data = i ;assert_eq!(*data, i );}) );
     }
     for w in workers {
         w.join().unwrap();
     }
 }
