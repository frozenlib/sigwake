use std::{
    any::Any,
    fmt,
    sync::{Arc, Weak},
    task::Waker,
};

/// `Box<dyn FnOnce()>` like wrapper with optional allocation-free alternatives.
#[derive(Debug)]
pub struct Action(RawAction);

impl Action {
    /// Create a new action from a `FnOnce`.
    pub fn new(f: impl FnOnce() + Sync + Send + 'static) -> Self {
        Self(RawAction::Fn(Box::new(f)))
    }

    /// Create a new action from an `Arc` and a function.
    ///
    /// If `f` is a ZST (Zero-Sized Type), this method does not allocate memory.
    pub fn from_arc_fn<T: Send + Sync + 'static>(
        this: Arc<T>,
        f: impl FnOnce(Arc<T>) + Sync + Send + Copy + 'static,
    ) -> Self {
        Self::from_arc_fn_usize(this, move |this, _| f(this), 0)
    }

    /// Create a new action from an `Arc` and a function with a parameter.
    ///
    /// If `f` is a ZST (Zero-Sized Type), this method does not allocate memory.
    pub fn from_arc_fn_usize<T: Send + Sync + 'static>(
        this: Arc<T>,
        f: impl FnOnce(Arc<T>, usize) + Sync + Send + Copy + 'static,
        param: usize,
    ) -> Self {
        Self(RawAction::Arc {
            this,
            f: Box::new(move |this, param| f(this.downcast().unwrap(), param)),
            param,
        })
    }

    /// Create a new action from an `Weak` and a function.
    ///
    /// Calls the function only if the specified weak reference is alive when cancelled.
    ///
    /// If `f` is a ZST (Zero-Sized Type), this method does not allocate memory.
    pub fn from_weak_fn<T: Send + Sync + 'static>(
        this: Weak<T>,
        f: impl FnOnce(Arc<T>) + Sync + Send + Copy + 'static,
    ) -> Self {
        Self::from_weak_fn_usize(this, move |this, _| f(this), 0)
    }

    /// Create a new action from an `Weak` and a function with a parameter.
    ///
    /// Calls the function only if the specified weak reference is alive when cancelled.
    ///
    /// If `f` is a ZST (Zero-Sized Type), this method does not allocate memory.
    pub fn from_weak_fn_usize<T: Send + Sync + 'static>(
        this: Weak<T>,
        f: impl FnOnce(Arc<T>, usize) + Sync + Send + Copy + 'static,
        param: usize,
    ) -> Self {
        Self(RawAction::Weak {
            this,
            f: Box::new(move |this, param| f(this.downcast().unwrap(), param)),
            param,
        })
    }

    /// Call the action.
    pub fn call(self) {
        self.0.call()
    }
}
impl From<Box<dyn FnOnce() + Sync + Send>> for Action {
    fn from(value: Box<dyn FnOnce() + Sync + Send>) -> Self {
        Self(RawAction::Fn(value))
    }
}
impl<F: FnOnce() + Sync + Send + 'static> From<Box<F>> for Action {
    fn from(value: Box<F>) -> Self {
        Self(RawAction::Fn(value))
    }
}

impl From<Waker> for Action {
    fn from(waker: Waker) -> Self {
        Self(RawAction::Waker(waker))
    }
}
impl From<&Waker> for Action {
    fn from(waker: &Waker) -> Self {
        Self(RawAction::Waker(waker.clone()))
    }
}

enum RawAction {
    Fn(Box<dyn FnOnce() + Sync + Send>),
    Waker(Waker),
    Arc {
        this: Arc<dyn Any + Sync + Send>,
        f: Box<dyn FnOnce(Arc<dyn Any + Send + Sync>, usize) + Sync + Send>,
        param: usize,
    },
    Weak {
        this: Weak<dyn Any + Sync + Send>,
        f: Box<dyn FnOnce(Arc<dyn Any + Send + Sync>, usize) + Sync + Send>,
        param: usize,
    },
}
impl RawAction {
    fn call(self) {
        match self {
            RawAction::Fn(f) => f(),
            RawAction::Waker(waker) => waker.wake(),
            RawAction::Arc { this, f, param } => f(this, param),
            RawAction::Weak { this, f, param } => {
                if let Some(this) = this.upgrade() {
                    f(this, param);
                }
            }
        }
    }
}
impl std::fmt::Debug for RawAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawAction::Fn(_) => write!(f, "Fn"),
            RawAction::Waker(_) => write!(f, "Waker"),
            RawAction::Arc { .. } => write!(f, "Arc"),
            RawAction::Weak { .. } => write!(f, "Weak"),
        }
    }
}
