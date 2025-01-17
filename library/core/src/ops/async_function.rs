use crate::future::Future;
use crate::marker::Tuple;

/// An async-aware version of the [`Fn`](crate::ops::Fn) trait.
///
/// All `async fn` and functions returning futures implement this trait.
#[unstable(feature = "async_fn_traits", issue = "none")]
#[rustc_paren_sugar]
#[fundamental]
#[must_use = "async closures are lazy and do nothing unless called"]
#[cfg_attr(not(bootstrap), lang = "async_fn")]
pub trait AsyncFn<Args: Tuple>: AsyncFnMut<Args> {
    /// Future returned by [`AsyncFn::async_call`].
    #[unstable(feature = "async_fn_traits", issue = "none")]
    type CallFuture<'a>: Future<Output = Self::Output>
    where
        Self: 'a;

    /// Call the [`AsyncFn`], returning a future which may borrow from the called closure.
    #[unstable(feature = "async_fn_traits", issue = "none")]
    extern "rust-call" fn async_call(&self, args: Args) -> Self::CallFuture<'_>;
}

/// An async-aware version of the [`FnMut`](crate::ops::FnMut) trait.
///
/// All `async fn` and functions returning futures implement this trait.
#[unstable(feature = "async_fn_traits", issue = "none")]
#[rustc_paren_sugar]
#[fundamental]
#[must_use = "async closures are lazy and do nothing unless called"]
#[cfg_attr(not(bootstrap), lang = "async_fn_mut")]
pub trait AsyncFnMut<Args: Tuple>: AsyncFnOnce<Args> {
    /// Future returned by [`AsyncFnMut::async_call_mut`].
    #[unstable(feature = "async_fn_traits", issue = "none")]
    type CallMutFuture<'a>: Future<Output = Self::Output>
    where
        Self: 'a;

    /// Call the [`AsyncFnMut`], returning a future which may borrow from the called closure.
    #[unstable(feature = "async_fn_traits", issue = "none")]
    extern "rust-call" fn async_call_mut(&mut self, args: Args) -> Self::CallMutFuture<'_>;
}

/// An async-aware version of the [`FnOnce`](crate::ops::FnOnce) trait.
///
/// All `async fn` and functions returning futures implement this trait.
#[unstable(feature = "async_fn_traits", issue = "none")]
#[rustc_paren_sugar]
#[fundamental]
#[must_use = "async closures are lazy and do nothing unless called"]
#[cfg_attr(not(bootstrap), lang = "async_fn_once")]
pub trait AsyncFnOnce<Args: Tuple> {
    /// Future returned by [`AsyncFnOnce::async_call_once`].
    #[unstable(feature = "async_fn_traits", issue = "none")]
    type CallOnceFuture: Future<Output = Self::Output>;

    /// Output type of the called closure's future.
    #[unstable(feature = "async_fn_traits", issue = "none")]
    type Output;

    /// Call the [`AsyncFnOnce`], returning a future which may move out of the called closure.
    #[unstable(feature = "async_fn_traits", issue = "none")]
    extern "rust-call" fn async_call_once(self, args: Args) -> Self::CallOnceFuture;
}

mod impls {
    use super::{AsyncFn, AsyncFnMut, AsyncFnOnce};
    use crate::future::Future;
    use crate::marker::Tuple;

    #[unstable(feature = "async_fn_traits", issue = "none")]
    impl<F: Fn<A>, A: Tuple> AsyncFn<A> for F
    where
        <F as FnOnce<A>>::Output: Future,
    {
        type CallFuture<'a> = <F as FnOnce<A>>::Output where Self: 'a;

        extern "rust-call" fn async_call(&self, args: A) -> Self::CallFuture<'_> {
            self.call(args)
        }
    }

    #[unstable(feature = "async_fn_traits", issue = "none")]
    impl<F: FnMut<A>, A: Tuple> AsyncFnMut<A> for F
    where
        <F as FnOnce<A>>::Output: Future,
    {
        type CallMutFuture<'a> = <F as FnOnce<A>>::Output where Self: 'a;

        extern "rust-call" fn async_call_mut(&mut self, args: A) -> Self::CallMutFuture<'_> {
            self.call_mut(args)
        }
    }

    #[unstable(feature = "async_fn_traits", issue = "none")]
    impl<F: FnOnce<A>, A: Tuple> AsyncFnOnce<A> for F
    where
        <F as FnOnce<A>>::Output: Future,
    {
        type CallOnceFuture = <F as FnOnce<A>>::Output;

        type Output = <<F as FnOnce<A>>::Output as Future>::Output;

        extern "rust-call" fn async_call_once(self, args: A) -> Self::CallOnceFuture {
            self.call_once(args)
        }
    }
}

mod internal_implementation_detail {
    /// A helper trait that is used to enforce that the `ClosureKind` of a goal
    /// is within the capabilities of a `CoroutineClosure`, and which allows us
    /// to delay the projection of the tupled upvar types until after upvar
    /// analysis is complete.
    ///
    /// The `Self` type is expected to be the `kind_ty` of the coroutine-closure,
    /// and thus either `?0` or `i8`/`i16`/`i32` (see docs for `ClosureKind`
    /// for an explanation of that). The `GoalKind` is also the same type, but
    /// representing the kind of the trait that the closure is being called with.
    #[cfg_attr(not(bootstrap), lang = "async_fn_kind_helper")]
    trait AsyncFnKindHelper<GoalKind> {
        // Projects a set of closure inputs (arguments), a region, and a set of upvars
        // (by move and by ref) to the upvars that we expect the coroutine to have
        // according to the `GoalKind` parameter above.
        //
        // The `Upvars` parameter should be the upvars of the parent coroutine-closure,
        // and the `BorrowedUpvarsAsFnPtr` will be a function pointer that has the shape
        // `for<'env> fn() -> (&'env T, ...)`. This allows us to represent the binder
        // of the closure's self-capture, and these upvar types will be instantiated with
        // the `'closure_env` region provided to the associated type.
        type Upvars<'closure_env, Inputs, Upvars, BorrowedUpvarsAsFnPtr>;
    }
}
