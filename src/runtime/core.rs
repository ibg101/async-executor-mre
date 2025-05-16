use std::{
    pin::Pin, 
    sync::{
        mpsc::{self, Receiver, Sender}, 
        Arc, 
        Mutex
    }
};
use super::waker::{ArcWake, waker_ref};
// use futures::task::{ArcWake, waker_ref};


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
            }
        }
    }
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    let (tx, rx) = mpsc::channel();
    (Executor { ready_queue: rx }, Spawner { tx })
}