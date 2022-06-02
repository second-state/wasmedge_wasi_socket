use crate::wasi_poll as poll;
use std::os::wasi::prelude::*;

#[derive(Clone)]
pub enum Subscription {
    Timeout {
        userdata: u64,
        timeout: std::time::SystemTime,
    },
    IO {
        userdata: u64,
        fd: RawFd,
        read_event: bool,
        write_event: bool,
    },
    TimeoutIO {
        userdata: u64,
        fd: RawFd,
        read_event: bool,
        write_event: bool,
        timeout: std::time::SystemTime,
    },
}

impl Subscription {
    pub fn timeout(userdata: u64, timeout: std::time::SystemTime) -> Self {
        Subscription::Timeout { userdata, timeout }
    }
    pub fn io<F: AsRawFd>(
        userdata: u64,
        fd: &F,
        read_event: bool,
        write_event: bool,
        timeout: Option<std::time::SystemTime>,
    ) -> Self {
        let fd = fd.as_raw_fd();
        if let Some(timeout) = timeout {
            Subscription::TimeoutIO {
                userdata,
                fd,
                read_event,
                write_event,
                timeout,
            }
        } else {
            Subscription::IO {
                userdata,
                fd,
                read_event,
                write_event,
            }
        }
    }
}

pub enum EventType {
    Timeout,
    Error(std::io::Error),
    Read,
    Write,
}

pub struct Event {
    pub event_type: EventType,
    pub userdata: u64,
}

fn to_subscription_vec(subs: &[Subscription]) -> Vec<poll::Subscription> {
    let mut fds = vec![];
    for s in subs {
        match s {
            Subscription::Timeout { userdata, timeout } => {
                let userdata = *userdata;
                let timeout = timeout.duration_since(std::time::UNIX_EPOCH).unwrap();
                let nanoseconds = timeout.as_nanos();
                let s = poll::Subscription {
                    userdata,
                    u: poll::SubscriptionU {
                        tag: poll::EVENTTYPE_CLOCK,
                        u: poll::SubscriptionUU {
                            clock: poll::SubscriptionClock {
                                id: poll::CLOCKID_REALTIME,
                                timeout: nanoseconds as u64,
                                precision: 0,
                                flags: poll::SUBCLOCKFLAGS_SUBSCRIPTION_CLOCK_ABSTIME,
                            },
                        },
                    },
                };
                fds.push(s);
            }
            Subscription::IO {
                userdata,
                fd,
                read_event,
                write_event,
            } => {
                let fd = *fd;
                let userdata = *userdata;
                if *read_event {
                    let s = poll::Subscription {
                        userdata,
                        u: poll::SubscriptionU {
                            tag: poll::EVENTTYPE_FD_READ,
                            u: poll::SubscriptionUU {
                                fd_read: poll::SubscriptionFdReadwrite {
                                    file_descriptor: fd as u32,
                                },
                            },
                        },
                    };
                    fds.push(s);
                }
                if *write_event {
                    let s = poll::Subscription {
                        userdata,
                        u: poll::SubscriptionU {
                            tag: poll::EVENTTYPE_FD_WRITE,
                            u: poll::SubscriptionUU {
                                fd_read: poll::SubscriptionFdReadwrite {
                                    file_descriptor: fd as u32,
                                },
                            },
                        },
                    };
                    fds.push(s);
                }
            }
            Subscription::TimeoutIO {
                userdata,
                fd,
                read_event,
                write_event,
                timeout,
            } => {
                let fd = *fd;
                let userdata = *userdata;
                if *read_event {
                    let s = poll::Subscription {
                        userdata,
                        u: poll::SubscriptionU {
                            tag: poll::EVENTTYPE_FD_READ,
                            u: poll::SubscriptionUU {
                                fd_read: poll::SubscriptionFdReadwrite {
                                    file_descriptor: fd as u32,
                                },
                            },
                        },
                    };
                    fds.push(s);
                }
                if *write_event {
                    let s = poll::Subscription {
                        userdata,
                        u: poll::SubscriptionU {
                            tag: poll::EVENTTYPE_FD_WRITE,
                            u: poll::SubscriptionUU {
                                fd_read: poll::SubscriptionFdReadwrite {
                                    file_descriptor: fd as u32,
                                },
                            },
                        },
                    };
                    fds.push(s);
                }

                {
                    let timeout = timeout.duration_since(std::time::UNIX_EPOCH).unwrap();
                    let nanoseconds = timeout.as_nanos();
                    let s = poll::Subscription {
                        userdata,
                        u: poll::SubscriptionU {
                            tag: poll::EVENTTYPE_CLOCK,
                            u: poll::SubscriptionUU {
                                clock: poll::SubscriptionClock {
                                    id: poll::CLOCKID_REALTIME,
                                    timeout: nanoseconds as u64,
                                    precision: 0,
                                    flags: poll::SUBCLOCKFLAGS_SUBSCRIPTION_CLOCK_ABSTIME,
                                },
                            },
                        },
                    };
                    fds.push(s);
                }
            }
        }
    }
    fds
}

pub fn poll(subs: &[Subscription]) -> std::io::Result<Vec<Event>> {
    use std::io;
    unsafe {
        let fds = to_subscription_vec(subs);
        let mut revent = vec![poll::Event::empty(); fds.len()];

        let n = poll::poll(fds.as_ptr(), revent.as_mut_ptr(), fds.len())?;

        let mut events = vec![];

        for i in 0..n {
            let event = revent[i];
            match event.type_ {
                poll::EVENTTYPE_CLOCK => {
                    events.push({
                        Event {
                            event_type: EventType::Timeout,
                            userdata: event.userdata,
                        }
                    });
                }
                poll::EVENTTYPE_FD_READ | poll::EVENTTYPE_FD_WRITE => {
                    if event.error > 0 {
                        let e = io::Error::from_raw_os_error(event.error as i32);
                        events.push(Event {
                            event_type: EventType::Error(e),
                            userdata: event.userdata,
                        });
                        continue;
                    }

                    if event.type_ == poll::EVENTTYPE_FD_READ {
                        events.push(Event {
                            event_type:EventType::Read,
                            userdata: event.userdata,
                        });
                    } else {
                        if event.fd_readwrite.flags & poll::EVENTRWFLAGS_FD_READWRITE_HANGUP > 0 {
                            let e = io::Error::new(io::ErrorKind::NotConnected, "POLLHUP");
                            events.push(Event {
                                event_type: EventType::Error(e),
                                userdata: event.userdata,
                            });
                        }else{
                            events.push(Event {
                                event_type:EventType::Write,
                                userdata: event.userdata,
                            });
                        }
                    };
                }
                _ => {}
            }
        }

        Ok(events)
    }
}
