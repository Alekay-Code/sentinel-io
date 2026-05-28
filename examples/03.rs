use sentinel::JoinHandle;
use sentinel_io as sentinel;
use std::{future::Future, task::Poll};
use sentinel_io::time::Timer;
use std::time::Duration;
// use sentinel_io::runtime;

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

// #[sentinel::main]
// async fn main() {
//     let c1 = Counter::new(1,5);
//
//     let h2 = sentinel::spawn(async {
//         let c2 = Counter::new(2, 10);
//         c2.await;
//     });
//
//     let h3 = sentinel::spawn(async {
//         let timer = Timer::new(Duration::from_secs(5));
//         timer.await;
//     });
//
//     c1.await;
//     h2.await;
//     h3.await;
// }

fn main() {
    let rt = sentinel::Runtime::new();
    let n = 10000;
    let count = 1000;
    let mut handlers = Vec::new();

    rt.block_on(async move {

        for i in 0..n {
            let counter = Counter::new(i.into(), count);

            let h = sentinel::spawn(async move {
                counter.await;
            });

            handlers.push(h);
        }

        for h in handlers {
            h.await;
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
