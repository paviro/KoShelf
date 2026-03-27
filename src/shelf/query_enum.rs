//! Declarative macros for generating `parse` / `as_str` on query-parameter enums.

/// Generate `parse(&str) -> Result<Self, (ApiErrorCode, String)>` and
/// `as_str(self) -> &'static str` for enums whose parse errors include a
/// field name and the list of valid values.
///
/// ```ignore
/// query_enum! {
///     pub enum MetricsGroupBy {
///         Total => "total",
///         Day   => "day",
///     }
///     field_name: "group_by"
/// }
/// ```
macro_rules! query_enum {
    (
        $vis:vis enum $Name:ident {
            $( $Variant:ident => $str:literal ),+ $(,)?
        }
        field_name: $field:literal
    ) => {
        impl $Name {
            pub fn parse(value: &str) -> Result<Self, (crate::server::api::responses::error::ApiErrorCode, String)> {
                match value {
                    $( $str => Ok(Self::$Variant), )+
                    _ => Err((
                        crate::server::api::responses::error::ApiErrorCode::InvalidQuery,
                        format!(
                            concat!("'", $field, "' must be one of: ", query_enum!(@join $($str),+), "; got '{}'"),
                            value,
                        ),
                    )),
                }
            }

            pub fn as_str(self) -> &'static str {
                match self {
                    $( Self::$Variant => $str, )+
                }
            }
        }
    };

    // Internal: build a comma-separated literal string from the variant strings.
    (@join $first:literal $(, $rest:literal)*) => {
        concat!($first $(, ", ", $rest)*)
    };
}

/// Generate `parse(&str) -> Result<Self, ApiErrorCode>` and
/// `as_str(self) -> &'static str` for enums whose parse errors carry no message.
macro_rules! query_enum_bare {
    (
        $vis:vis enum $Name:ident {
            $( $Variant:ident => $str:literal ),+ $(,)?
        }
    ) => {
        impl $Name {
            pub fn parse(value: &str) -> Result<Self, crate::server::api::responses::error::ApiErrorCode> {
                match value {
                    $( $str => Ok(Self::$Variant), )+
                    _ => Err(crate::server::api::responses::error::ApiErrorCode::InvalidQuery),
                }
            }

            pub fn as_str(self) -> &'static str {
                match self {
                    $( Self::$Variant => $str, )+
                }
            }
        }
    };
}

