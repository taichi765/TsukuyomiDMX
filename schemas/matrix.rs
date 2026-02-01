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
#[doc = "`PixelNumberConstraint`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^=[1-9][0-9]*$\","]
#[doc = "      \"$comment\": \"exact position\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^>=[1-9][0-9]*$\","]
#[doc = "      \"$comment\": \"minimum position\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^<=[1-9][0-9]*$\","]
#[doc = "      \"$comment\": \"maximum position\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[1-9][0-9]*n$\","]
#[doc = "      \"$comment\": \"position divisible by number\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"pattern\": \"^[1-9][0-9]*n\\\\+[1-9][0-9]*$\","]
#[doc = "      \"$comment\": \"position divisible by number with remainder\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"const\": \"even\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"const\": \"odd\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum PixelNumberConstraint {
    Variant0(PixelNumberConstraintVariant0),
    Variant1(PixelNumberConstraintVariant1),
    Variant2(PixelNumberConstraintVariant2),
    Variant3(PixelNumberConstraintVariant3),
    Variant4(PixelNumberConstraintVariant4),
    Variant5(::serde_json::Value),
    Variant6(::serde_json::Value),
}
impl ::std::convert::From<&Self> for PixelNumberConstraint {
    fn from(value: &PixelNumberConstraint) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant0> for PixelNumberConstraint {
    fn from(value: PixelNumberConstraintVariant0) -> Self {
        Self::Variant0(value)
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant1> for PixelNumberConstraint {
    fn from(value: PixelNumberConstraintVariant1) -> Self {
        Self::Variant1(value)
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant2> for PixelNumberConstraint {
    fn from(value: PixelNumberConstraintVariant2) -> Self {
        Self::Variant2(value)
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant3> for PixelNumberConstraint {
    fn from(value: PixelNumberConstraintVariant3) -> Self {
        Self::Variant3(value)
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant4> for PixelNumberConstraint {
    fn from(value: PixelNumberConstraintVariant4) -> Self {
        Self::Variant4(value)
    }
}
#[doc = "`PixelNumberConstraintArray`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"array\","]
#[doc = "  \"items\": {"]
#[doc = "    \"$ref\": \"#/definitions/pixelNumberConstraint\""]
#[doc = "  },"]
#[doc = "  \"minItems\": 1,"]
#[doc = "  \"uniqueItems\": true"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct PixelNumberConstraintArray(pub Vec<PixelNumberConstraint>);
impl ::std::ops::Deref for PixelNumberConstraintArray {
    type Target = Vec<PixelNumberConstraint>;
    fn deref(&self) -> &Vec<PixelNumberConstraint> {
        &self.0
    }
}
impl ::std::convert::From<PixelNumberConstraintArray> for Vec<PixelNumberConstraint> {
    fn from(value: PixelNumberConstraintArray) -> Self {
        value.0
    }
}
impl ::std::convert::From<&PixelNumberConstraintArray> for PixelNumberConstraintArray {
    fn from(value: &PixelNumberConstraintArray) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<Vec<PixelNumberConstraint>> for PixelNumberConstraintArray {
    fn from(value: Vec<PixelNumberConstraint>) -> Self {
        Self(value)
    }
}
#[doc = "`PixelNumberConstraintVariant0`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^=[1-9][0-9]*$\","]
#[doc = "  \"$comment\": \"exact position\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PixelNumberConstraintVariant0(::std::string::String);
impl ::std::ops::Deref for PixelNumberConstraintVariant0 {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant0> for ::std::string::String {
    fn from(value: PixelNumberConstraintVariant0) -> Self {
        value.0
    }
}
impl ::std::convert::From<&PixelNumberConstraintVariant0> for PixelNumberConstraintVariant0 {
    fn from(value: &PixelNumberConstraintVariant0) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for PixelNumberConstraintVariant0 {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^=[1-9][0-9]*$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^=[1-9][0-9]*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PixelNumberConstraintVariant0 {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PixelNumberConstraintVariant0 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PixelNumberConstraintVariant0 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PixelNumberConstraintVariant0 {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`PixelNumberConstraintVariant1`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^>=[1-9][0-9]*$\","]
#[doc = "  \"$comment\": \"minimum position\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PixelNumberConstraintVariant1(::std::string::String);
impl ::std::ops::Deref for PixelNumberConstraintVariant1 {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant1> for ::std::string::String {
    fn from(value: PixelNumberConstraintVariant1) -> Self {
        value.0
    }
}
impl ::std::convert::From<&PixelNumberConstraintVariant1> for PixelNumberConstraintVariant1 {
    fn from(value: &PixelNumberConstraintVariant1) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for PixelNumberConstraintVariant1 {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^>=[1-9][0-9]*$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^>=[1-9][0-9]*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PixelNumberConstraintVariant1 {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PixelNumberConstraintVariant1 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PixelNumberConstraintVariant1 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PixelNumberConstraintVariant1 {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`PixelNumberConstraintVariant2`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^<=[1-9][0-9]*$\","]
#[doc = "  \"$comment\": \"maximum position\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PixelNumberConstraintVariant2(::std::string::String);
impl ::std::ops::Deref for PixelNumberConstraintVariant2 {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant2> for ::std::string::String {
    fn from(value: PixelNumberConstraintVariant2) -> Self {
        value.0
    }
}
impl ::std::convert::From<&PixelNumberConstraintVariant2> for PixelNumberConstraintVariant2 {
    fn from(value: &PixelNumberConstraintVariant2) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for PixelNumberConstraintVariant2 {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^<=[1-9][0-9]*$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^<=[1-9][0-9]*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PixelNumberConstraintVariant2 {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PixelNumberConstraintVariant2 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PixelNumberConstraintVariant2 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PixelNumberConstraintVariant2 {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`PixelNumberConstraintVariant3`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[1-9][0-9]*n$\","]
#[doc = "  \"$comment\": \"position divisible by number\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PixelNumberConstraintVariant3(::std::string::String);
impl ::std::ops::Deref for PixelNumberConstraintVariant3 {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant3> for ::std::string::String {
    fn from(value: PixelNumberConstraintVariant3) -> Self {
        value.0
    }
}
impl ::std::convert::From<&PixelNumberConstraintVariant3> for PixelNumberConstraintVariant3 {
    fn from(value: &PixelNumberConstraintVariant3) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for PixelNumberConstraintVariant3 {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| ::regress::Regex::new("^[1-9][0-9]*n$").unwrap());
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[1-9][0-9]*n$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PixelNumberConstraintVariant3 {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PixelNumberConstraintVariant3 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PixelNumberConstraintVariant3 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PixelNumberConstraintVariant3 {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`PixelNumberConstraintVariant4`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[1-9][0-9]*n\\\\+[1-9][0-9]*$\","]
#[doc = "  \"$comment\": \"position divisible by number with remainder\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PixelNumberConstraintVariant4(::std::string::String);
impl ::std::ops::Deref for PixelNumberConstraintVariant4 {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<PixelNumberConstraintVariant4> for ::std::string::String {
    fn from(value: PixelNumberConstraintVariant4) -> Self {
        value.0
    }
}
impl ::std::convert::From<&PixelNumberConstraintVariant4> for PixelNumberConstraintVariant4 {
    fn from(value: &PixelNumberConstraintVariant4) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for PixelNumberConstraintVariant4 {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| {
                ::regress::Regex::new("^[1-9][0-9]*n\\+[1-9][0-9]*$").unwrap()
            });
        if PATTERN.find(value).is_none() {
            return Err("doesn't match pattern \"^[1-9][0-9]*n\\+[1-9][0-9]*$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for PixelNumberConstraintVariant4 {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PixelNumberConstraintVariant4 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PixelNumberConstraintVariant4 {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for PixelNumberConstraintVariant4 {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
