use db_key::Key;


#[derive(Debug,Clone,Eq,PartialEq, Hash)]
pub struct MyKey(pub String);
impl Key for MyKey{
    fn from_u8(key: &[u8]) -> MyKey {
        MyKey(std::str::from_utf8(key).unwrap().to_string())
      }
    
    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.0.as_bytes())
      }
}

impl MyKey {
    pub fn from_string(v : String) -> Self {
        Self(v)
    }
}