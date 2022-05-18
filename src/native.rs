use std::rc::Rc;
use std::time::SystemTime;

use crate::value::*;

pub struct NativeClock {}

impl NativeFunc for NativeClock {
    fn call(&self, _arg_count: usize, _args: &[Rc<Value>]) -> Value {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => Value::Number(n.as_millis() as f64),
            Err(_) => panic!("can't get system time"),
        }
    }
}
