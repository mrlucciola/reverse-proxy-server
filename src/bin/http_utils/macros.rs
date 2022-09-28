#![macro_use]

// pub trait Display {}
// impl fmt::Display for dyn Display {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             p => write!(f, "{:?}", p),
//         }
//     }
// }

#[macro_export]
macro_rules! impl_display {
    (for $($t:ty),+) => {
        $(impl std::fmt::Display for $t {
            // fn double(&self) -> u32 {
            //     self.x * 2
            // }
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    p => write!(f, "{:?}", p),
                    // ConnectionError::ParseError(parse_int_error) => write!(f, "{}", parse_int_error),
                    // ConnectionError::IoError(io_error) => write!(f, "{}", io_error),
                }
            }
        })*
    }
}

pub(crate) use impl_display;
