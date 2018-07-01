use std::collections::HashMap;
use std::fmt;

pub trait MappingCache: fmt::Debug {
    fn get(&self, key: String,) -> Option<&Vec<String>>;
    fn put(&mut self, key: String, value: Vec<String>) -> Result<String, String>;
}

pub struct FiloCache {
    cache_size: i64,
    mappings: HashMap<String, Vec<String>>,
    mapping_stack: Vec<String>,
}

impl FiloCache {
    pub fn new(cache_size: i64) -> FiloCache {
        if cache_size == 0 {
            FiloCache {
                cache_size: 0,
                mappings: HashMap::new(),
                mapping_stack: Vec::new(),
            }
        } else {
            FiloCache {
                cache_size: cache_size,
                mappings: HashMap::with_capacity(cache_size as usize),
                mapping_stack: Vec::with_capacity(cache_size as usize),
            }
        }
    }
}

impl MappingCache for FiloCache {
    fn get(&self, key: String) -> Option<&Vec<String>> {
        self.mappings.get(&key)
    }

    fn put(&mut self, key: String, value: Vec<String>) -> Result<String, String> {
        if self.mappings.len() >= self.cache_size as usize {
            let old_key = self.mapping_stack.remove(0);
            let _ = self.mappings.remove(&old_key);
        }

        self.mappings.insert(key.clone(), value);
        self.mapping_stack.push(key);

        Err(String::from("Blah"))
    }
}

impl fmt::Debug for FiloCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.mappings)
    }
}
