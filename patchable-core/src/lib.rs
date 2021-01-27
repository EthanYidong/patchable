//! Trait(s) for patchable structs in Rust.
//! You probably want [`patchable`](https://docs.rs/patchable) instead, for derive functionality.

/// Trait to indicate that a type can be patched.
pub trait Patchable<P> {
    fn apply_patch(&mut self, patch: P);
}

impl<T> Patchable<Option<T>> for T {
    fn apply_patch(&mut self, patch: Option<T>) {
        if let Some(inner) = patch {
            *self = inner;
        }
    }
}

impl<T, P> Patchable<Vec<P>> for T where T: Patchable<P> {
    fn apply_patch(&mut self, patches: Vec<P>) {
        for patch in patches {
            self.apply_patch(patch);
        }
    }
}
