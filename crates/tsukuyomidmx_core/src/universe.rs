#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct UniverseId(u8);

impl UniverseId {
    pub fn new(v: u8) -> Self {
        Self(v)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for UniverseId {
    fn default() -> Self {
        Self(1)
    }
}

/// DmxAddress with bound 1..=512.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DmxAddress(usize);

impl DmxAddress {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(512);

    /// Returns `None` when value is out of bounds (1..=512).
    /// # Examples
    /// ```
    /// use tsukuyomi_core::universe::DmxAddress;
    ///
    /// let address1 = DmxAddress::new(0);
    /// assert!(address1.is_none());
    ///
    /// let address2 = DmxAddress::new(512);
    /// assert!(address2.is_some());
    /// ```
    pub fn new(value: usize) -> Option<Self> {
        if Self::MIN.0 <= value && value <= Self::MAX.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> usize {
        self.0
    }

    pub fn checked_add(self, rhs: usize) -> Option<DmxAddress> {
        DmxAddress::new(self.0 + rhs)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<usize> {
        self.0.checked_sub(rhs.0)
    }
}

impl Default for DmxAddress {
    fn default() -> Self {
        Self(1)
    }
}
