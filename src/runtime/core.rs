use std::{
    pin::Pin, 
    sync::{
        mpsc::{self, Receiver, Sender}, 
        Arc, 
        Mutex
    }
};
use super::waker::{ArcWake, waker_ref};


pub struct Executor {
    ready_queue: Receiver<Arc<Task>>
}

struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
    tx: Sender<Arc<Task>>
}

pub struct Spawner {
    tx: Sender<Arc<Task>>
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) -> () {
        let task = Arc::clone(arc_self);
        arc_self.tx
            .send(task)
            .expect("Failed to place the Task onto the queue!");
    }
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) -> () {
        let boxed_future = Box::pin(future);
        let task = Arc::new(Task {
            future: Mutex::new(Some(boxed_future)),
            tx: self.tx.clone()
        });
        self.tx.send(task).expect("Failed to place the Task onto the queue!");
    }
}

impl Executor {
    pub fn run(&self) -> () {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_lock = task.future.lock().unwrap();
            if let Some(mut future) = future_lock.take() {
                std::mem::drop(future_lock);
                
                let waker = waker_ref(&task);
                let mut context = std::task::Context::from_waker(&waker);

                if future.as_mut().poll(&mut context).is_pending() {
                    *task.future.lock().unwrap() = Some(future);
                }

                // At the end of the scope we have the following:
                // - strong count of Arc<Task> = 2 (task + clone of the waker inside poll method)
                // - task drops => -1 = 1
                // - waker drops => -1 = 0 (even though the waker creation doesn't contain calls to Arc::from_raw(ptr),
                //   dropping which will cause a strong ref count decreasing,
                //   BUT instead i have implemented RawWakerExtended::drop() method, which is automatically called on drop of the `waker` here.
                //   And the `RawWakerExtended::drop()` implementation contains temporary recreation of the Arc from the pointer to Arc<Task>, which
                //   is dropped later => leading to decreasing of the strong ref count.
                // 
                // This can be validated by: 
                //   1. commenting out the Arc::from_raw(ptr) in RawWakerExtended::drop()
                // OR
                //   2. wrapping Waker during it's creation inside ManuallyDrop()  
                // OR
                //   3. if additional cloning is accepted, use Arc::clone() + Arc::into_raw() instead of  Arc::as_ptr() in the waker_ref()
            }
        }
    }
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    let (tx, rx) = mpsc::channel();
    (Executor { ready_queue: rx }, Spawner { tx })
}