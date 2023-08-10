use crate::context::Context;
use std::collections::HashMap;

/*
Here, we use static Option<HashMap<..>> instead of static HashMap<..>

Error usage below:
static router:HashMap<String,fn(Context)> = HashMap::new();
*/
static mut ROUTER: Option<HashMap<String, RoutingItem>> = None;

pub struct RoutingItem {
    pub path: String,
    pub method: String,
    pub func: fn(Context),
}

pub fn insert(key: &str, value: RoutingItem) {
    unsafe {
        if let None = ROUTER {
            ROUTER = Some(HashMap::new());
        }
        ROUTER.as_mut().unwrap().insert(key.to_string(),value);
    }
}

pub fn find(key: &str) -> Option<&'static RoutingItem> {
    unsafe {
        if let None = ROUTER {
            ROUTER = Some(HashMap::new());
        }
        return ROUTER.as_ref().unwrap().get(key);
    }
}
