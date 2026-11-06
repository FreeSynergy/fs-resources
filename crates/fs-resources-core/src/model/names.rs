//! A macro for enums that have a fixed wire name per variant.
//!
//! Pattern: Template (macro) — one definition gives the enum its serde form and `as_str`, so the
//! stored text, the index value and the error text of a state can never differ.

macro_rules! named_enum {
    ($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $v:ident = $s:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $( $(#[$vm])* #[serde(rename = $s)] $v ),+
        }

        impl $name {
            /// The name used in stored content, index fields and error texts.
            pub fn as_str(self) -> &'static str {
                match self { $( Self::$v => $s ),+ }
            }
        }
    };
}
