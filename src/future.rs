use std::{
    pin::Pin,
    task::{Poll, Context, Waker},
    sync::{
        Arc,
        RwLock,
    }
};

#[derive(Default)]
pub struct Sleep {
    state: Arc<RwLock<State>>
}

#[derive(Default)]
struct State {
    is_ready: bool,
    waker: Option<Waker>
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.read().unwrap().is_ready {
            Poll::Ready(())
        } else {
            self.state.write().unwrap().waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub fn sleep(dur: std::time::Duration) -> Sleep {
    let sleep = Sleep::default();
    let sleep_state = Arc::clone(&sleep.state);

    std::thread::spawn(move || {
        std::thread::sleep(dur);
        let mut state_lock = sleep_state.write().unwrap();
        state_lock.is_ready = true;
        if let Some(waker) = state_lock.waker.take() {
            waker.wake();
        }
    });

    sleep
}