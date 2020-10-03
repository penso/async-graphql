//! Field guards

use crate::{Context, Result};
use serde::export::PhantomData;

/// Field guard
///
/// Guard is a pre-condition for a field that is resolved if `Ok(())` is returned, otherwise an error is returned.
///
/// This trait is defined through the [`async-trait`](https://crates.io/crates/async-trait) macro.
#[async_trait::async_trait]
pub trait Guard {
    /// Check whether the guard will allow access to the field.
    async fn check(&self, ctx: &Context<'_>) -> Result<()>;
}

/// An extension trait for `Guard`.
pub trait GuardExt: Guard + Sized {
    /// Perform `and` operator on two rules
    fn and<R: Guard>(self, other: R) -> And<Self, R> {
        And(self, other)
    }

    /// Perform `or` operator on two rules
    fn or<R: Guard>(self, other: R) -> Or<Self, R> {
        Or(self, other)
    }
}

impl<T: Guard> GuardExt for T {}

/// Guard for [`GuardExt::and`](trait.GuardExt.html#method.and).
pub struct And<A: Guard, B: Guard>(A, B);

#[async_trait::async_trait]
impl<A: Guard + Send + Sync, B: Guard + Send + Sync> Guard for And<A, B> {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        self.0.check(ctx).await.and(self.1.check(ctx).await)
    }
}

/// Guard for [`GuardExt::or`](trait.GuardExt.html#method.or).
pub struct Or<A: Guard, B: Guard>(A, B);

#[async_trait::async_trait]
impl<A: Guard + Send + Sync, B: Guard + Send + Sync> Guard for Or<A, B> {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        self.0.check(ctx).await.or(self.1.check(ctx).await)
    }
}

/// Field post guard
///
/// This is a post-condition for a field that is resolved if `Ok(()` is returned, otherwise an error is returned.
///
/// This trait is defined through the [`async-trait`](https://crates.io/crates/async-trait) macro.
#[async_trait::async_trait]
pub trait PostGuard<T: Send + Sync> {
    /// Check whether to allow the result of the field through.
    async fn check(&self, ctx: &Context<'_>, result: &T) -> Result<()>;
}

/// An extension trait for `PostGuard<T>`
pub trait PostGuardExt<T: Send + Sync>: PostGuard<T> + Sized {
    /// Merge the two guards.
    fn and<R: PostGuard<T>>(self, other: R) -> PostAnd<T, Self, R> {
        PostAnd(self, other, PhantomData)
    }
}

impl<T: PostGuard<R>, R: Send + Sync> PostGuardExt<R> for T {}

/// PostGuard for [`PostGuardExt<T>::and`](trait.PostGuardExt.html#method.and).
pub struct PostAnd<T: Send + Sync, A: PostGuard<T>, B: PostGuard<T>>(A, B, PhantomData<T>);

#[async_trait::async_trait]
impl<T: Send + Sync, A: PostGuard<T> + Send + Sync, B: PostGuard<T> + Send + Sync> PostGuard<T>
    for PostAnd<T, A, B>
{
    async fn check(&self, ctx: &Context<'_>, result: &T) -> Result<()> {
        self.0.check(ctx, result).await?;
        self.1.check(ctx, result).await
    }
}
