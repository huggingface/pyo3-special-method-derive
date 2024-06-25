//! This crate exports 2 traits which should be implemented for
//! every type for which its field or variant is not skipped.
//!
//! It also exports a macro to use the Debug and Display traits to generate a PyDebug and PyDisplay
//! implementation.

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
        format!(
            "[{}]",
            self.iter()
                .map(|x| x.fmt_debug())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
impl<T: PyDisplay> PyDisplay for &[T] {
    fn fmt_display(&self) -> String {
        format!(
            "[{}]",
            self.iter()
                .map(|x| x.fmt_display())
                .collect::<Vec<_>>()
                .join(",")
        )
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
