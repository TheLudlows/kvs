use std::mem::{size_of, size_of_val};
use std::ptr;



#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

lazy_static! {
    pub static ref i:Box<usize> = Box::new(0);
}

static  j:usize = 0;
#[test]
fn test_ptr() {

    let new_vec:Vec<String> = vec![String::new(), String::new(), String::new()];
    unsafe {
        let ptr1 = new_vec.as_ptr() as * mut String;
        let _ = ptr::replace(ptr1, String::from("abc"));

        let ptr2 = ptr1.add(1);
        let _ = ptr::replace(ptr2, String::from("def"));
    }

    println!("{:?}", new_vec);



    let ptr = (i.as_ref() as *const usize) as *mut usize;
    unsafe {
        ptr.write(100);
    }
    println!("{}", *i);

}