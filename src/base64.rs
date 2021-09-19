use std::fmt;

/// Data that is passed in as URL safe Base64
pub struct Base64Data(pub Vec<u8>);

impl fmt::Debug for Base64Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() > 10 {
            write!(
                f,
                "Base64Data([{}, {}, {}, ...{} more bytes)",
                self.0[0],
                self.0[1],
                self.0[2],
                self.0.len() - 3,
            )
        } else {
            write!(f, "Base64Data({:?})", self.0)
        }
    }
}
