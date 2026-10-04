// Lesson 6: raw pointers and unsafe.
//
// A raw pointer (`*const T`, `*mut T`) is an address with no owner and no
// guarantees: it may be null, dangling, or point at the wrong type. Making
// one is safe. Reading through one is `unsafe`: the compiler can't check it,
// so YOU promise it's valid. This lesson shows how to make that promise
// carefully, and ends by building a tiny Rc to show how the real one works.

use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

// ---- 1. The original example, and why it was wrong ---------------------------------------
//
//     fn get_value_from_raw_pointer(raw_p: *const u32) -> u32 {
//         unsafe { *raw_p }
//     }
//
// This is a SAFE function (no `unsafe` in its signature), so anyone may call
// it with any pointer: `get_value_from_raw_pointer(std::ptr::null())` is
// undefined behaviour, from safe code. That's exactly what Rust's rules forbid.
// Made `pub`, Clippy rejects it:
//     error: this public function might dereference a raw pointer but is not marked `unsafe`
//
// Three correct alternatives:

/// Fix 1, almost always the best: don't use a raw pointer. A reference is
/// guaranteed to be valid, so no `unsafe` is needed at all.
fn read_value(value: &u32) -> u32 {
    *value
}

/// Fix 2: an `unsafe fn`. The `unsafe` moves the responsibility to the caller,
/// and the `# Safety` section says exactly what they must guarantee.
///
/// # Safety
/// `pointer` must be non-null, properly aligned, and point to an initialised
/// `u32` that stays valid for the duration of the call.
unsafe fn read_raw(pointer: *const u32) -> u32 {
    // SAFETY: the caller guarantees `pointer` is valid (see above).
    unsafe { *pointer }
}

/// Fix 3, a tempting half-fix: checking for null. Null is only ONE way a
/// pointer can be invalid: it can also dangle, or be misaligned. So this
/// still has to be `unsafe`, and `as_ref` says so.
///
/// # Safety
/// `pointer` must be either null, or valid as for `read_raw`.
unsafe fn read_if_not_null(pointer: *const u32) -> Option<u32> {
    // SAFETY: the caller guarantees that a non-null `pointer` is valid.
    unsafe { pointer.as_ref() }.copied()
}

// ---- 2. Pointer arithmetic: what a slice does inside ----------------------------------------

/// Sums a slice by walking a raw pointer through it, as slice iteration does
/// inside (with the bounds check that makes it safe).
fn sum_with_pointers(numbers: &[u32]) -> u32 {
    let start = numbers.as_ptr();
    let mut total = 0;
    for i in 0..numbers.len() {
        // SAFETY: `i < numbers.len()`, so `start.add(i)` stays inside the slice,
        // and the slice keeps its memory valid for as long as we borrow it.
        total += unsafe { *start.add(i) };
    }
    total
}

// ---- 3. How Rc works inside: a minimal version ----------------------------------------------

/// What lives on the heap: the count and the value, together.
struct Inner<T> {
    count: Cell<usize>,
    value: T,
}

/// A minimal reference-counted pointer, like `Rc` (lesson 3), built from a raw
/// pointer. `NonNull` is a `*mut` that's never null; `PhantomData` tells the
/// compiler this struct logically owns an `Inner<T>`.
///
/// `NonNull` is neither `Send` nor `Sync`, so `MyRc` can't cross threads,
/// exactly like `Rc`: its plain counter isn't thread-safe (lesson 4).
pub struct MyRc<T> {
    pointer: NonNull<Inner<T>>,
    _owns: PhantomData<Inner<T>>,
}

impl<T> MyRc<T> {
    pub fn new(value: T) -> Self {
        // Put the Inner on the heap, then take over ownership as a raw pointer:
        // from now on, OUR code is responsible for freeing it.
        let inner = Box::new(Inner { count: Cell::new(1), value });
        MyRc { pointer: NonNull::from(Box::leak(inner)), _owns: PhantomData }
    }

    fn inner(&self) -> &Inner<T> {
        // SAFETY: `pointer` came from a Box, and the Inner is only freed when
        // the count reaches 0. While `self` exists, the count is at least 1.
        unsafe { self.pointer.as_ref() }
    }

    pub fn count(this: &Self) -> usize {
        this.inner().count.get()
    }
}

impl<T> Clone for MyRc<T> {
    fn clone(&self) -> Self {
        let count = &self.inner().count;
        count.set(count.get() + 1); // one more owner; the value isn't copied
        MyRc { pointer: self.pointer, _owns: PhantomData }
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner().value
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        let count = &self.inner().count;
        count.set(count.get() - 1);
        if count.get() == 0 {
            // SAFETY: the count is 0, so this was the last owner: nothing else can
            // use the Inner any more. It was created by `Box::new`, so turning it
            // back into a Box and dropping that frees it correctly, exactly once.
            unsafe { drop(Box::from_raw(self.pointer.as_ptr())) };
        }
    }
}

fn main() {
    println!("1. Reading a value: a reference, or an unsafe raw pointer");
    let value: u32 = 99;
    println!("    through a reference: {}", read_value(&value));
    let pointer: *const u32 = &raw const value; // making a raw pointer is safe
    // SAFETY: `pointer` points to `value`, which is alive and a valid u32.
    println!("    through a raw pointer: {}", unsafe { read_raw(pointer) });
    // SAFETY: null is allowed, and a non-null pointer here points to `value`.
    println!("    null checked: {:?}, {:?}", unsafe { read_if_not_null(pointer) }, unsafe {
        read_if_not_null(std::ptr::null())
    });

    println!("\n2. Pointer arithmetic");
    println!("    sum of [1, 2, 3, 4] by walking a pointer: {}", sum_with_pointers(&[1, 2, 3, 4]));

    println!("\n3. A homemade Rc");
    let first = MyRc::new(String::from("shared text"));
    let second = MyRc::clone(&first);
    println!("    {:?} has {} owners", *second, MyRc::count(&first));
    drop(second);
    println!("    after dropping one: {} owner", MyRc::count(&first));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn all_the_readers_agree() {
        let value = 7u32;
        assert_eq!(read_value(&value), 7);
        // SAFETY: points to a live u32.
        assert_eq!(unsafe { read_raw(&value) }, 7);
        // SAFETY: null, or a pointer to a live u32.
        unsafe {
            assert_eq!(read_if_not_null(&value), Some(7));
            assert_eq!(read_if_not_null(std::ptr::null()), None);
        }
    }

    #[test]
    fn pointer_arithmetic_matches_iteration() {
        let numbers: Vec<u32> = (1..=100).collect();
        assert_eq!(sum_with_pointers(&numbers), numbers.iter().sum());
        assert_eq!(sum_with_pointers(&[]), 0);
    }

    #[test]
    fn my_rc_counts_and_shares() {
        let a = MyRc::new(vec![1, 2, 3]);
        let b = a.clone();
        assert_eq!(MyRc::count(&a), 2);
        assert_eq!(*b, [1, 2, 3]); // Deref
        assert!(std::ptr::eq(&*a, &*b)); // the same value, not a copy
        drop(a);
        assert_eq!(MyRc::count(&b), 1);
    }

    /// Records when it's dropped, to prove the value is freed exactly once.
    struct DropCounter(Rc<RefCell<u32>>);
    impl Drop for DropCounter {
        fn drop(&mut self) {
            *self.0.borrow_mut() += 1;
        }
    }

    #[test]
    fn my_rc_frees_the_value_exactly_once_after_the_last_owner() {
        let drops = Rc::new(RefCell::new(0));
        let a = MyRc::new(DropCounter(Rc::clone(&drops)));
        let b = a.clone();
        let c = b.clone();
        drop(a);
        drop(c);
        assert_eq!(*drops.borrow(), 0); // `b` still owns it
        drop(b);
        assert_eq!(*drops.borrow(), 1); // freed now, and only once
    }
}
