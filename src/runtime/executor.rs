use crate::poll::{poll, Event, EventType, Subscription};
use futures::lock::Mutex;
use futures::task::{self, ArcWake, Waker};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::io;
use std::os::wasi::prelude::RawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;

scoped_tls::scoped_thread_local!(pub(crate) static EXECUTOR: Executor);

struct Poller {
    subs: HashMap<i32, Subscription>,
}

impl Poller {
    fn add(&mut self, fd: RawFd) {
        self.subs.insert(
            fd,
            Subscription::IO {
                userdata: 0,
                fd: fd,
                read_event: false,
                write_event: false,
            },
        );
    }
    fn delete(&mut self, fd: RawFd) {
        self.subs.remove(&fd);
    }
    fn modify(&mut self, fd: RawFd, interest: Interest) {
        self.subs.entry(fd).and_modify(|e| {
            *e = match interest {
                Interest::Read => Subscription::IO {
                    userdata: fd as u64,
                    fd: fd,
                    read_event: true,
                    write_event: false,
                },
                Interest::Write => Subscription::IO {
                    userdata: fd as u64,
                    fd: fd,
                    read_event: false,
                    write_event: true,
                },
            };
        });
    }
    fn poll(&self) -> Vec<Event> {
        poll(
            &(self
                .subs
                .clone()
                .into_values()
                .collect::<Vec<Subscription>>()),
        )
        .unwrap()
    }
}

pub enum Interest {
    Read,
    Write,
}

pub struct TaskQueue {
    queue: RefCell<VecDeque<Arc<Task>>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        const DEFAULT_TASK_QUEUE_SIZE: usize = 4096;
        Self::new_with_capacity(DEFAULT_TASK_QUEUE_SIZE)
    }
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            queue: RefCell::new(VecDeque::with_capacity(capacity)),
        }
    }

    pub(crate) fn push(&self, runnable: Arc<Task>) {
        // println!("add task");
        self.queue.borrow_mut().push_back(runnable);
    }

    pub(crate) fn pop(&self) -> Option<Arc<Task>> {
        // println!("remove task");
        self.queue.borrow_mut().pop_front()
    }
}

pub struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &std::sync::Arc<Self>) {
        EXECUTOR.with(|ex| ex.tasks.push(arc_self.clone()));
    }
}

pub struct Reactor {
    poll: Poller,
    wakers_map: HashMap<u64, Waker>,
}

impl Reactor {
    pub fn new() -> Self {
        Self {
            poll: Poller {
                subs: HashMap::new(),
            },
            wakers_map: HashMap::new(),
        }
    }
    pub fn wait(&mut self) {
        let events = self.poll.poll();
        for event in events {
            let token = event.userdata;
            let waker = match event.event_type {
                EventType::Read => self.wakers_map.remove(&(token * 2)),
                EventType::Write => self.wakers_map.remove(&(token * 2 + 1)),
                _ => None,
            }
            .unwrap();
            waker.wake();
        }
    }
    pub fn add(&mut self, fd: RawFd) -> io::Result<()> {
        self.poll.add(fd);
        Ok(())
    }

    pub fn delete(&mut self, fd: RawFd) {
        self.wakers_map.remove(&(fd as u64 * 2));
        self.wakers_map.remove(&(fd as u64 * 2 + 1));
        self.poll.delete(fd);
    }

    pub fn modify(&mut self, fd: RawFd, interest: Interest, cx: &mut Context) {
        match interest {
            Interest::Read => {
                self.wakers_map.insert(fd as u64 * 2, cx.waker().clone());
            }
            Interest::Write => {
                self.wakers_map
                    .insert(fd as u64 * 2 + 1, cx.waker().clone());
            }
        }
        self.poll.modify(fd, interest)
    }
}

pub struct Executor {
    tasks: TaskQueue,
    pub reactor: RefCell<Reactor>,
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    let task = Task {
        future: Mutex::new(Box::pin(future)),
    };
    EXECUTOR.with(|ex| {
        ex.tasks.push(Arc::new(task));
    });
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: TaskQueue::new(),
            reactor: RefCell::new(Reactor::new()),
        }
    }

    pub fn block_on<F, T, O>(&mut self, f: F) -> O
    where
        F: Fn() -> T,
        T: Future<Output = O> + 'static,
    {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        EXECUTOR.set(self, || {
            let mut fut = f();
            let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
            loop {
                // return if the outer future is ready
                if let std::task::Poll::Ready(t) = fut.as_mut().poll(&mut cx) {
                    break t;
                }

                // consume all tasks
                while let Some(t) = self.tasks.pop() {
                    let future = t.future.try_lock().unwrap();
                    let w = task::waker(t.clone());
                    let mut context = Context::from_waker(&w);
                    let _ = Pin::new(future).as_mut().poll(&mut context);
                }

                // no task to execute now, it may ready
                if let std::task::Poll::Ready(t) = fut.as_mut().poll(&mut cx) {
                    break t;
                }

                // block for io
                self.reactor.borrow_mut().wait();
            }
        })
    }
}
