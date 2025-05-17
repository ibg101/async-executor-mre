use std::{
    sync::Arc,
    task::{RawWaker, RawWakerVTable, Waker}
};


pub trait ArcWake {
    fn wake(self: Arc<Self>) -> () {
        Self::wake_by_ref(&self);
    }

    fn wake_by_ref(arc_self: &Arc<Self>) -> ();
}

pub struct RawWakerExtended;

// https://docs.rs/futures-task/0.3.31/src/futures_task/waker.rs.html
impl RawWakerExtended {
    unsafe fn clone<T: ArcWake>(ptr: *const ()) -> RawWaker {
        println!("DEBUG clone!");
        unsafe { Arc::increment_strong_count(ptr) };
        RawWaker::new(ptr, waker_vtable::<T>())
    }

    unsafe fn drop<T: ArcWake>(ptr: *const ()) {
        println!("DEBUG drop!");
        unsafe { Arc::from_raw(ptr) };
    }

    unsafe fn wake<T: ArcWake>(ptr: *const ()) {
        println!("DEBUG wake!");
        let data_from_ptr = unsafe { Arc::from_raw(ptr as *const T) };
        ArcWake::wake(data_from_ptr);
    }

    unsafe fn wake_by_ref<T: ArcWake>(ptr: *const ()) {
        println!("DEBUG wake by ref!");
        let data_from_ptr = unsafe { Arc::from_raw(ptr as *const T) };
        ArcWake::wake_by_ref(&data_from_ptr);
    }
}

fn waker_vtable<T: ArcWake>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        RawWakerExtended::clone::<T>, 
        RawWakerExtended::wake::<T>, 
        RawWakerExtended::wake_by_ref::<T>, 
        RawWakerExtended::drop::<T>
    )
}

pub fn waker_ref<T: ArcWake>(arc_waker: &Arc<T>) -> Waker {
    let ptr = Arc::as_ptr(arc_waker) as *const ();

    // if uncomment this will work

    // let cloned = arc_waker.clone();
    // let ptr = Arc::into_raw(cloned) as *const ();

    let raw_waker = RawWaker::new(ptr, waker_vtable::<T>());
    unsafe { Waker::from_raw(raw_waker) }
}


// if uncomment this will work as well

// pub struct WakerRef(std::mem::ManuallyDrop<Waker>);
// impl std::ops::Deref for WakerRef {
//     type Target = Waker;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// pub fn waker_ref<W: ArcWake>(wake: &Arc<W>) -> WakerRef {
//     let ptr = Arc::as_ptr(wake) as *const ();
//     let waker = std::mem::ManuallyDrop::new(
//         unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<W>())) }
//     );
//     WakerRef(waker)
// }