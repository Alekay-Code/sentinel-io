use std::time::{Duration, Instant};
use std::{future::Future, task::Poll};
use tokio;

struct Counter {
    id: i32,
    current: usize,
    to: usize,
}

impl Counter {
    fn new(id: i32, to: usize) -> Self {
        Self { id, current: 0, to }
    }
}

impl Future for Counter {
    type Output = usize;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        println!("[{}] counter: {}", self.id, self.current);
        self.current += 1;

        if self.current >= self.to {
            println!("[{}] counter finish: {}", self.id, self.current);
            return Poll::Ready(self.current);
        } else {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
    }
}

struct Timer {
    id: usize,
    start: Instant,
    duration: Duration,
}

impl Timer {
    fn new(id: usize, duration: Duration) -> Self {
        Self {
            id,
            start: Instant::now(),
            duration,
        }
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.start.elapsed() < self.duration {
            println!(
                "Timer[{}] poll -> {}s",
                self.id,
                self.start.elapsed().as_secs()
            );
            return Poll::Pending;
        } else {
            println!(
                "Timer[{}] finish -> {}/{}s",
                self.id,
                self.duration.as_secs(),
                self.start.elapsed().as_secs()
            );
            return Poll::Ready(());
        }
    }
}

// #[tokio::main]
// async fn main() {
//     let c1 = Box::pin(Counter::new(1, 5));
//     let c2 = Box::pin(Counter::new(2, 7));
//     // let c3 = Box::pin(Counter::new(3, 10));
//     // let t1 = Box::pin(Timer::new(1, Duration::from_secs(2)));
//     // let t2 = Box::pin(Timer::new(2, Duration::from_secs(20)));
//
//     let h1 = tokio::spawn(async move {
//         return c1.await;
//         // c2.await;
//         // t1.await;
//     });
//
//     let h2 = tokio::spawn(async move {
//         c2.await;
//     });
//
//     let res = h1.await;
//     println!("RES: {}", res.unwrap());
//     _ = h2.await;
// }

fn main() {
    // let rt = tokio::runtime::Runtime::new().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    let n = 10000;
    let count = 1000;
    let mut handlers = Vec::new();

    rt.block_on(async move {

        for i in 0..n {
            let counter = Counter::new(i.into(), count);

            let h = tokio::spawn(async move {
                counter.await;
            });

            handlers.push(h);
        }

        for h in handlers {
            let _ = h.await;
        }

        // let c1 = Counter::new(1, 5);
        // let count = c1.await;
        //
        // let h = sentinel::spawn(async {
        //     let timer = Timer::new(Duration::from_secs(5));
        //     let dur = timer.await;
        //     return dur;
        // });
        //
        // println!("Counter: {count}");
        // let dur = h.await;
        // println!("Timer finish in {}", dur.as_secs());
    });
}
