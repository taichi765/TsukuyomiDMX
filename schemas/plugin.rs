#![allow(clippy::redundant_closure_call)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::clone_on_copy)]

#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
#[doc = "`FileLocations`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"minProperties\": 1,"]
#[doc = "  \"properties\": {"]
#[doc = "    \"main\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"user\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileLocations {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub main: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub user: ::std::option::Option<::std::string::String>,
}
impl ::std::convert::From<&FileLocations> for FileLocations {
    fn from(value: &FileLocations) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for FileLocations {
    fn default() -> Self {
        Self {
            main: Default::default(),
            user: Default::default(),
        }
    }
}
impl FileLocations {
    pub fn builder() -> builder::FileLocations {
        Default::default()
    }
}
#[doc = "HTML string lines, will be joined by \\n."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"HTML string lines, will be joined by \\\\n.\","]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"type\": \"string\""]
#[doc = "  },"]
#[doc = "  \"minItems\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct HtmlStringLines(pub ::std::vec::Vec<::std::string::String>);
impl ::std::ops::Deref for HtmlStringLines {
    type Target = ::std::vec::Vec<::std::string::String>;
    fn deref(&self) -> &::std::vec::Vec<::std::string::String> {
        &self.0
    }
}
impl ::std::convert::From<HtmlStringLines> for ::std::vec::Vec<::std::string::String> {
    fn from(value: HtmlStringLines) -> Self {
        value.0
    }
}
impl ::std::convert::From<&HtmlStringLines> for HtmlStringLines {
    fn from(value: &HtmlStringLines) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<::std::vec::Vec<::std::string::String>> for HtmlStringLines {
    fn from(value: ::std::vec::Vec<::std::string::String>) -> Self {
        Self(value)
    }
}
#[doc = "`UrlString`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"format\": \"uri\","]
#[doc = "  \"pattern\": \"^(ftp|http|https)://[^ \\\"]+$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
#[serde(transparent)]
pub struct UrlString(pub ::std::string::String);
impl ::std::ops::Deref for UrlString {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<UrlString> for ::std::string::String {
    fn from(value: UrlString) -> Self {
        value.0
    }
}
impl ::std::convert::From<&UrlString> for UrlString {
    fn from(value: &UrlString) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<::std::string::String> for UrlString {
    fn from(value: ::std::string::String) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for UrlString {
    type Err = ::std::convert::Infallible;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}
impl ::std::fmt::Display for UrlString {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct FileLocations {
        main: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        user: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for FileLocations {
        fn default() -> Self {
            Self {
                main: Ok(Default::default()),
                user: Ok(Default::default()),
            }
        }
    }
    impl FileLocations {
        pub fn main<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.main = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for main: {}", e));
            self
        }
        pub fn user<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.user = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for user: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<FileLocations> for super::FileLocations {
        type Error = super::error::ConversionError;
        fn try_from(
            value: FileLocations,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                main: value.main?,
                user: value.user?,
            })
        }
    }
    impl ::std::convert::From<super::FileLocations> for FileLocations {
        fn from(value: super::FileLocations) -> Self {
            Self {
                main: Ok(value.main),
                user: Ok(value.user),
            }
        }
    }
}
