use wasm_bindgen::prelude::*;

macro_rules! impl_Storage {
    ($name:ident, $get:expr, $docname:expr) => {
        #[doc = "Access to the "]
        #[doc = $docname]
        #[doc = " storage."]
        pub mod $name {
            use crate::UnwrapJsVal;
            use wasm_bindgen::prelude::*;
            fn get_storage() -> crate::Storage {
                let storage = $get
                    .throw_err()
                    .expect_throw(concat!($docname, " storage not available"));
                crate::Storage::new(storage)
            }

            /// Get the key for the `idx`th item in the storage.
            ///
            /// Prefer using `get`.
            pub fn key(idx: usize) -> Option<String> {
                get_storage().key(idx)
            }

            /// Get a record from the storage if present.
            pub fn get(key: &str) -> Option<String> {
                get_storage().get(key)
            }

            /// Set a record in the storage and return the old record with the same key, if
            /// present.
            pub fn set(key: &str, val: &str) -> Option<String> {
                get_storage().set(key, val)
            }

            /// Remove a record from the storage and return it, if present.
            pub fn remove(key: &str) -> Option<String> {
                get_storage().remove(key)
            }

            /// Remove all records from the storage.
            pub fn clear() {
                get_storage().clear()
            }

            /// An iterator over key/value pairs, in the order they were last modified (newest first) (I
            /// think).
            ///
            /// Editing the storage while iterating will invalidate the iterator.
            pub fn iter() -> impl Iterator<Item = (String, String)> {
                get_storage().into_iter()
            }

            /// Get the number of records in the storage.
            pub fn count() -> usize {
                get_storage().count()
            }
        }
    };
}

impl_Storage!(local, crate::window().local_storage(), "local");
impl_Storage!(session, crate::window().session_storage(), "session");

#[derive(Debug, Clone)]
struct Storage {
    inner: web_sys::Storage,
}

impl Storage {
    fn new(inner: web_sys::Storage) -> Self {
        // Casts only work when sizeof(usize) > sizeof(u32), which is ok on wasm.
        debug_assert!(std::mem::size_of::<usize>() >= std::mem::size_of::<u32>());
        Storage { inner }
    }

    /// Get the nth key from an index n.
    fn key(&self, idx: usize) -> Option<String> {
        // futureproof for 64 bit wasm
        if idx > u32::max_value() as usize {
            wasm_bindgen::throw_str("u32 overflow on position");
        }
        self.inner.key(idx as u32).throw_err()
    }

    fn get(&self, key: &str) -> Option<String> {
        self.inner.get_item(key).throw_err()
    }

    fn set(&self, key: &str, val: &str) -> Option<String> {
        let old = self.get(key);
        self.inner.set_item(key, val).throw_err();
        old
    }

    fn remove(&self, key: &str) -> Option<String> {
        let old = self.get(key);
        self.inner.remove_item(key).throw_err();
        old
    }

    fn clear(&self) {
        self.inner.clear().throw_err()
    }

    /// Get the number of records in the storage.
    fn count(&self) -> usize {
        self.inner.length().throw_err() as usize
    }
}

impl IntoIterator for Storage {
    type Item = (String, String);
    type IntoIter = StorageIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        StorageIntoIter::new(self)
    }
}

struct StorageIntoIter {
    position: usize,
    inner: Storage,
}

impl StorageIntoIter {
    fn new(inner: Storage) -> Self {
        StorageIntoIter { position: 0, inner }
    }
}

impl Iterator for StorageIntoIter {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.inner.key(self.position)?;
        let value = self.inner.get(&key).unwrap_throw();
        self.position += 1;
        Some((key, value))
    }
}

/// Get the window or throw
fn window() -> web_sys::Window {
    web_sys::window().expect_throw("`window` global not available - are we on a web platform?")
}

/// Rethrow exceptions from js land.
trait UnwrapJsVal<T> {
    fn throw_err(self) -> T;
}

impl<T> UnwrapJsVal<T> for Result<T, JsValue> {
    fn throw_err(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => wasm_bindgen::throw_val(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    fn test_storage<Type>(storage: crate::Storage<Type>) {
        assert_eq!(storage.count(), 0);
        assert!(storage.set("first", "a_val").is_none());
        assert_eq!(storage.count(), 1);
        assert_eq!(storage.get("first"), Some("a_val".into()));
        assert_eq!(storage.set("first", "another_val"), Some("a_val".into()));
        assert_eq!(storage.count(), 1);
        assert!(storage.set("second", "val_3").is_none());
        assert_eq!(storage.count(), 2);
        let mut iter = storage.iter();
        assert_eq!(iter.next(), Some(("second".into(), "val_3".into())));
        assert_eq!(iter.next(), Some(("first".into(), "another_val".into())));
        assert_eq!(iter.next(), None);
        assert_eq!(storage.remove("first"), Some("another_val".into()));
        let mut iter = storage.into_iter();
        assert_eq!(iter.next(), Some(("second".into(), "val_3".into())));
        assert_eq!(iter.next(), None);
    }

    #[wasm_bindgen_test]
    fn local() {
        test_storage(crate::local());
    }

    #[wasm_bindgen_test]
    fn session() {
        test_storage(crate::session());
    }
}
