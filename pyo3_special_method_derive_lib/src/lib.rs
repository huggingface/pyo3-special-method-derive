pub trait PyDebug {
    fn fmt_debug(&self) -> String;
}

impl<T: PyDebug> PyDebug for Option<T> {
    fn fmt_debug(&self) -> String {
        match self {
            Some(x) => x.fmt_debug(),
            None => "None".to_string(),
        }
    }
}

pub trait PyDisplay {
    fn fmt_display(&self) -> String;
}

impl<T: PyDisplay> PyDisplay for Option<T> {
    fn fmt_display(&self) -> String {
        match self {
            Some(x) => x.fmt_display(),
            None => "None".to_string(),
        }
    }
}

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

pydebug_pydisplay!(u32);
pydebug_pydisplay!(f32);
pydebug_pydisplay!(String);
