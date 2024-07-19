//! This crate exports 2 traits which should be implemented for
//! every type for which its field or variant is not skipped.
//!
//! It also exports a macro to use the Debug and Display traits to generate a PyDebug and PyDisplay
//! implementation.

use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
};

/// Number of elements to print for each type with an implementation in this crate,
/// defaults to 100.
pub static ELLIPSIS_N: AtomicUsize = AtomicUsize::new(100);

/// Types which can be displayed into the `__repr__` implementation.
pub trait PyDebug {
    fn fmt_debug(&self) -> String;
}

/// Types which can be displayed into the `__str__` implementation.
pub trait PyDisplay {
    fn fmt_display(&self) -> String;
}

/// Use this trait to automatically derive PyDebug and PyDisplay for your type.
/// It uses the Debug and Display traits internally. Because this usage can expose
/// Rust semantics, types, or otherwise look foreign, this should only be used for types which
/// are simple enough to not be distinctly Rust-y.
#[macro_export]
macro_rules! pydebug_pydisplay {
    ($t:ty) => {
        impl PyDebug for $t {
            fn fmt_debug(&self) -> String {
                format!("{self:?}")
            }
        }
        impl PyDisplay for $t {
            fn fmt_display(&self) -> String {
                format!("{self:?}")
                // NOTE: Do not use the Display impl.
                // format!("{self}")
            }
        }
    };
}

pydebug_pydisplay!(u8);
pydebug_pydisplay!(u16);
pydebug_pydisplay!(u32);
pydebug_pydisplay!(u64);
pydebug_pydisplay!(u128);

pydebug_pydisplay!(i8);
pydebug_pydisplay!(i16);
pydebug_pydisplay!(i32);
pydebug_pydisplay!(i64);
pydebug_pydisplay!(i128);

pydebug_pydisplay!(f32);
pydebug_pydisplay!(f64);

pydebug_pydisplay!(bool);

pydebug_pydisplay!(String);
pydebug_pydisplay!(&str);

impl<T: PyDebug> PyDebug for &[T] {
    fn fmt_debug(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self.iter().map(|x| x.fmt_debug()).take(n);
        if self.len() <= n {
            format!("[{}]", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("[{}, ...]", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<T: PyDisplay> PyDisplay for &[T] {
    fn fmt_display(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self.iter().map(|x| x.fmt_display()).take(n);
        if self.len() <= n {
            format!("[{}]", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("[{}, ...]", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<T: PyDebug> PyDebug for Vec<T> {
    fn fmt_debug(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self.iter().map(|x| x.fmt_debug()).take(n);
        if self.len() <= n {
            format!("[{}]", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("[{}, ...]", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<T: PyDisplay> PyDisplay for Vec<T> {
    fn fmt_display(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self.iter().map(|x| x.fmt_display()).take(n);
        if self.len() <= n {
            format!("[{}]", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("[{}, ...]", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<T: PyDebug> PyDebug for Option<T> {
    fn fmt_debug(&self) -> String {
        match self {
            Some(x) => x.fmt_debug(),
            None => "None".to_string(),
        }
    }
}

impl<T: PyDisplay> PyDisplay for Option<T> {
    fn fmt_display(&self) -> String {
        match self {
            Some(x) => x.fmt_display(),
            None => "None".to_string(),
        }
    }
}

impl<T: PyDebug> PyDebug for RwLock<T> {
    fn fmt_debug(&self) -> String {
        match self.read() {
            Ok(x) => x.fmt_debug(),
            Err(_) => "None".to_string(),
        }
    }
}

impl<T: PyDisplay> PyDisplay for RwLock<T> {
    fn fmt_display(&self) -> String {
        match self.read() {
            Ok(x) => x.fmt_display(),
            Err(_) => "None".to_string(),
        }
    }
}

impl<T: PyDebug> PyDebug for Mutex<T> {
    fn fmt_debug(&self) -> String {
        match self.lock() {
            Ok(x) => x.fmt_debug(),
            Err(_) => "None".to_string(),
        }
    }
}

impl<T: PyDisplay> PyDisplay for Mutex<T> {
    fn fmt_display(&self) -> String {
        match self.lock() {
            Ok(x) => x.fmt_display(),
            Err(_) => "None".to_string(),
        }
    }
}

impl<T: PyDebug> PyDebug for Arc<RwLock<T>> {
    fn fmt_debug(&self) -> String {
        match self.read() {
            Ok(x) => x.fmt_debug(),
            Err(_) => "None".to_string(),
        }
    }
}

impl<T: PyDisplay> PyDisplay for Arc<RwLock<T>> {
    fn fmt_display(&self) -> String {
        match self.read() {
            Ok(x) => x.fmt_display(),
            Err(_) => "None".to_string(),
        }
    }
}

impl<T: PyDebug> PyDebug for Arc<Mutex<T>> {
    fn fmt_debug(&self) -> String {
        match self.lock() {
            Ok(x) => x.fmt_debug(),
            Err(_) => "None".to_string(),
        }
    }
}

impl<T: PyDisplay> PyDisplay for Arc<Mutex<T>> {
    fn fmt_display(&self) -> String {
        match self.lock() {
            Ok(x) => x.fmt_display(),
            Err(_) => "None".to_string(),
        }
    }
}

impl<T: PyDebug + Copy> PyDebug for Cell<T> {
    fn fmt_debug(&self) -> String {
        self.get().fmt_debug()
    }
}

impl<T: PyDisplay + Copy> PyDisplay for Cell<T> {
    fn fmt_display(&self) -> String {
        self.get().fmt_display()
    }
}

impl<K: PyDebug, V: PyDebug> PyDebug for HashMap<K, V> {
    fn fmt_debug(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self
            .iter()
            .map(|(k, v)| format!("{}: {}", k.fmt_debug(), v.fmt_debug()))
            .take(n);
        if self.len() <= n {
            format!("{{{}}}", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("{{{}, ...}}", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<K: PyDisplay, V: PyDisplay> PyDisplay for HashMap<K, V> {
    fn fmt_display(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self
            .iter()
            .map(|(k, v)| format!("{}: {}", k.fmt_display(), v.fmt_display()))
            .take(n);
        if self.len() <= n {
            format!("{{{}}}", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("{{{}, ...}}", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<V: PyDebug> PyDebug for HashSet<V> {
    fn fmt_debug(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self.iter().map(|v| v.fmt_debug()).take(n);
        if self.len() <= n {
            format!("{{{}}}", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("{{{}, ...}}", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<V: PyDisplay> PyDisplay for HashSet<V> {
    fn fmt_display(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self.iter().map(|v| v.fmt_display()).take(n);
        if self.len() <= n {
            format!("{{{}}}", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("{{{}, ...}}", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<K: PyDebug, V: PyDebug> PyDebug for BTreeMap<K, V> {
    fn fmt_debug(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self
            .iter()
            .map(|(k, v)| format!("{}: {}", k.fmt_debug(), v.fmt_debug()))
            .take(n);
        if self.len() <= n {
            format!("{{{}}}", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("{{{}, ...}}", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<K: PyDisplay, V: PyDisplay> PyDisplay for BTreeMap<K, V> {
    fn fmt_display(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self
            .iter()
            .map(|(k, v)| format!("{}: {}", k.fmt_display(), v.fmt_display()))
            .take(n);
        if self.len() <= n {
            format!("{{{}}}", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("{{{}, ...}}", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<V: PyDebug> PyDebug for BTreeSet<V> {
    fn fmt_debug(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self.iter().map(|v| v.fmt_debug()).take(n);
        if self.len() <= n {
            format!("{{{}}}", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("{{{}, ...}}", map.collect::<Vec<_>>().join(", "))
        }
    }
}

impl<V: PyDisplay> PyDisplay for BTreeSet<V> {
    fn fmt_display(&self) -> String {
        let n = ELLIPSIS_N.load(Ordering::Relaxed);
        let map = self.iter().map(|v| v.fmt_display()).take(n);
        if self.len() <= n {
            format!("{{{}}}", map.collect::<Vec<_>>().join(", "))
        } else {
            format!("{{{}, ...}}", map.collect::<Vec<_>>().join(", "))
        }
    }
}
