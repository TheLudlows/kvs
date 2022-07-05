use db_key::Key;


#[derive(Debug,Clone,Eq,PartialEq, Hash)]
pub struct MyKey(pub Vec<u8>);
impl Key for MyKey{
    fn from_u8(key: &[u8]) -> MyKey {
        MyKey(key.to_vec())
      }
    
    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.0)
      }
}

impl MyKey {
    pub fn from_string(v : String) -> Self {
        Self(v.into_bytes())
    }
}