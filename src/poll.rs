use crate::AsRawFd;
use std::{collections::HashMap, os::wasi::prelude::*};
use wasi::{
    poll_oneoff, Errno, Event as WasiEvent, EventFdReadwrite, Subscription,
    SubscriptionFdReadwrite, SubscriptionU, SubscriptionUU, ERRNO_SUCCESS, EVENTTYPE_CLOCK,
    EVENTTYPE_FD_READ, EVENTTYPE_FD_WRITE,
};

trait AsPollFd {
    fn push_sub(&self, subs: &mut Vec<Subscription>);
}

// Associates readiness events with FD
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token(pub usize);

impl Token {
    /// Increases the token for next FD and return previous one
    pub fn add(&mut self) -> Token {
        let current = Token(self.0);
        self.0 += 1;
        current
    }
}

/// Token and corresponding event status
#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub token: Token,
    status: Interest,
}

impl Event {
    pub fn is_readable(&self) -> bool {
        self.status.is_readable()
    }

    pub fn is_writable(&self) -> bool {
        self.status.is_writable()
    }
}

/// Poll Instance
#[derive(Clone, Debug)]
pub struct Poll {
    tokens: HashMap<i32, Token>,
    selector: Selector,
}

impl Default for Poll {
    fn default() -> Self {
        Poll::new()
    }
}

impl Poll {
    /// Create a new poll instance
    pub fn new() -> Self {
        Poll {
            tokens: HashMap::new(),
            selector: Selector::new(),
        }
    }

    /// Register a file descriptor with the Poll instance
    pub fn register<T>(&mut self, fd: &T, token: Token, interest: Interest)
    where
        T: AsRawFd,
    {
        let poll_fd = PollFd {
            fd: fd.as_raw_fd() as i32,
            interest,
        };
        self.selector.poll_fds.insert(poll_fd.fd, poll_fd);
        self.tokens.insert(poll_fd.fd, token);
    }

    /// Reregister a file descriptor with the Poll instance
    pub fn reregister<T>(&mut self, fd: &T, token: Token, interest: Interest)
    where
        T: AsRawFd,
    {
        self.unregister(fd);
        let poll_fd = PollFd {
            fd: fd.as_raw_fd() as i32,
            interest,
        };
        self.selector.poll_fds.insert(poll_fd.fd, poll_fd);
        self.tokens.insert(poll_fd.fd, token);
    }

    /// Unregister a file descriptor with the Poll instance
    pub fn unregister<T>(&mut self, fd: &T)
    where
        T: AsRawFd,
    {
        self.selector.poll_fds.remove(&(fd.as_raw_fd() as i32));
    }

    /// Wait for readiness events
    ///
    /// Returns a vector of readiness events or error.
    pub fn poll(&mut self) -> Result<Vec<Event>, Errno> {
        if self.selector.poll_fds.is_empty() {
            return Ok(vec![]);
        }
        let mut subs = Vec::with_capacity(self.selector.poll_fds.len());
        for fd in self.selector.poll_fds.values() {
            fd.push_sub(&mut subs);
        }
        let mut oevents = vec![
            WasiEvent {
                userdata: 0,
                error: ERRNO_SUCCESS,
                type_: EVENTTYPE_CLOCK,
                fd_readwrite: EventFdReadwrite {
                    nbytes: 0,
                    flags: 0,
                },
            };
            subs.len()
        ];
        let _ = unsafe { poll_oneoff(subs.as_ptr(), oevents.as_mut_ptr(), subs.len()) }?;
        let mut status_map: HashMap<Token, u8> = HashMap::new();
        let mut events = Vec::new();
        for (_, event) in oevents.into_iter().enumerate() {
            let token = self.tokens.get(&(event.userdata as i32)).unwrap();
            if let Some(status) = status_map.get_mut(token) {
                *status |= event.type_.raw();
            } else {
                status_map.insert(*token, event.type_.raw());
            }
        }
        for (token, status) in status_map.into_iter() {
            events.push(Event {
                token,
                status: Interest::from_bits(status).unwrap(),
            });
        }
        Ok(events)
    }
}

/// Structs for containting poll fds
#[derive(Clone, Debug)]
pub struct Selector {
    poll_fds: HashMap<i32, PollFd>,
}

impl Selector {
    pub fn new() -> Selector {
        Selector {
            poll_fds: HashMap::new(),
        }
    }
}

impl Default for Selector {
    fn default() -> Self {
        Self::new()
    }
}

/// Interest used in registering
///
/// Currently only support read, write, and both(read and write).
#[derive(Copy, Clone, Debug)]
pub enum Interest {
    Read,
    Write,
    Both,
}

impl Interest {
    pub fn from_bits(bits: u8) -> Option<Interest> {
        match bits {
            1 => Some(Interest::Read),
            2 => Some(Interest::Write),
            3 => Some(Interest::Both),
            _ => None,
        }
    }

    pub fn is_readable(&self) -> bool {
        matches!(self, Interest::Read | Interest::Both)
    }

    pub fn is_writable(&self) -> bool {
        matches!(self, Interest::Write | Interest::Both)
    }
}

/// Pollable file descriptor
#[derive(Copy, Clone, Debug)]
pub struct PollFd {
    fd: RawFd,
    interest: Interest,
}

impl AsPollFd for PollFd {
    fn push_sub(&self, subs: &mut Vec<Subscription>) {
        match self.interest {
            Interest::Read => {
                subs.push(Subscription {
                    userdata: self.fd as u64,
                    u: SubscriptionU {
                        tag: EVENTTYPE_FD_READ.raw(),
                        u: SubscriptionUU {
                            fd_read: SubscriptionFdReadwrite {
                                file_descriptor: self.fd as u32,
                            },
                        },
                    },
                });
            }
            Interest::Write => {
                subs.push(Subscription {
                    userdata: self.fd as u64,
                    u: SubscriptionU {
                        tag: EVENTTYPE_FD_WRITE.raw(),
                        u: SubscriptionUU {
                            fd_read: SubscriptionFdReadwrite {
                                file_descriptor: self.fd as u32,
                            },
                        },
                    },
                });
            }
            Interest::Both => {
                subs.push(Subscription {
                    userdata: self.fd as u64,
                    u: SubscriptionU {
                        tag: EVENTTYPE_FD_WRITE.raw(),
                        u: SubscriptionUU {
                            fd_read: SubscriptionFdReadwrite {
                                file_descriptor: self.fd as u32,
                            },
                        },
                    },
                });
                subs.push(Subscription {
                    userdata: self.fd as u64,
                    u: SubscriptionU {
                        tag: EVENTTYPE_FD_READ.raw(),
                        u: SubscriptionUU {
                            fd_read: SubscriptionFdReadwrite {
                                file_descriptor: self.fd as u32,
                            },
                        },
                    },
                });
            }
        }
    }
}
