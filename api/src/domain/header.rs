use serde::{Deserialize, Serialize};

// TODO - should this be updated to hold an "active" field to hold that state instead of converting
// from Vec<(bool, String, String)>?
#[derive(
  Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug, sqlx::Encode, sqlx::Decode,
)]
pub struct Header {
  pub key: String,
  pub value: String,
}
#[derive(
  Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug, sqlx::Encode, sqlx::Decode,
)]
pub struct Headers(pub Vec<Header>);
impl FromIterator<(String, String)> for Headers {
  fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
    let mut h = Headers(Vec::new());
    for (k, v) in iter {
      h.0.push(Header { key: k, value: v });
    }
    h
  }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct HeadersIterator<'a> {
  headers: &'a [Header],
  index: usize,
}

impl<'a> Iterator for HeadersIterator<'a> {
  type Item = &'a Header;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index < self.headers.len() {
      let result = &self.headers[self.index];
      self.index += 1;
      Some(result)
    } else {
      None
    }
  }
}

impl<'a> IntoIterator for &'a Headers {
  type Item = &'a Header;
  type IntoIter = HeadersIterator<'a>;

  fn into_iter(self) -> Self::IntoIter {
    HeadersIterator {
      headers: &self.0,
      index: 0,
    }
  }
}
