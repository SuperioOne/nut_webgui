use core::{
  mem::MaybeUninit,
  num::NonZeroUsize,
  ops::{Deref, DerefMut},
};
use std::sync::Arc;
use tokio::sync::{AcquireError, Mutex, Semaphore, SemaphorePermit};

pub trait ItemAllocator {
  type Output;
  type Error;

  fn init(&self) -> impl Future<Output = Result<Self::Output, Self::Error>>;
  fn dealloc(&self, item: Self::Output) -> impl Future<Output = ()>;
}

pub enum ItemPoolError<E>
where
  E: Sized,
{
  PoolClosed,
  AllocatorError { inner: E },
}

impl<E> From<AcquireError> for ItemPoolError<E>
where
  E: Sized,
{
  fn from(_: AcquireError) -> Self {
    Self::PoolClosed
  }
}

struct InnerPool<T, A>
where
  A: ItemAllocator<Output = T>,
{
  items: Mutex<Vec<T>>,
  permits: Semaphore,
  allocator: A,
}

pub struct ItemPool<T, A>
where
  A: ItemAllocator<Output = T>,
{
  inner: Arc<InnerPool<T, A>>,
}

pub struct PoolGuard<'a, T, A>
where
  A: ItemAllocator<Output = T>,
{
  pool: Arc<InnerPool<T, A>>,
  _permit: SemaphorePermit<'a>,
  item: MaybeUninit<T>,
}

impl<T, A> Clone for ItemPool<T, A>
where
  A: ItemAllocator<Output = T>,
{
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

unsafe impl<T, G> Send for ItemPool<T, G> where G: ItemAllocator<Output = T> {}
unsafe impl<T, G> Sync for ItemPool<T, G> where G: ItemAllocator<Output = T> {}

impl<T, A> PoolGuard<'_, T, A>
where
  A: ItemAllocator<Output = T>,
{
  pub async fn release(mut self) {
    let item: T = unsafe { self.item.assume_init_read() };
    self.item = MaybeUninit::zeroed();
    self.pool.release(item).await;
  }
}

impl<T, A> Deref for PoolGuard<'_, T, A>
where
  A: ItemAllocator<Output = T>,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { self.item.assume_init_ref() }
  }
}

impl<T, A> DerefMut for PoolGuard<'_, T, A>
where
  A: ItemAllocator<Output = T>,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.item.assume_init_mut() }
  }
}

impl<T, A> ItemPool<T, A>
where
  A: ItemAllocator<Output = T>,
{
  pub fn new(limit: NonZeroUsize, allocator: A) -> Self {
    let limit: usize = limit.into();

    Self {
      inner: Arc::new(InnerPool {
        items: Mutex::new(Vec::with_capacity(limit)),
        permits: Semaphore::new(limit),
        allocator,
      }),
    }
  }

  pub async fn get(&self) -> Result<PoolGuard<T, A>, ItemPoolError<A::Error>> {
    let permit = self.inner.permits.acquire().await?;
    let mut items_lock = self.inner.items.lock().await;

    match items_lock.pop() {
      Some(item) => Ok(PoolGuard {
        _permit: permit,
        item: MaybeUninit::new(item),
        pool: self.inner.clone(),
      }),
      None => match self.inner.allocator.init().await {
        Ok(item) => Ok(PoolGuard {
          _permit: permit,
          item: MaybeUninit::new(item),
          pool: self.inner.clone(),
        }),
        Err(inner) => Err(ItemPoolError::AllocatorError { inner }),
      },
    }
  }

  pub async fn close(self) {
    self.inner.permits.close();

    let mut items = self.inner.items.lock().await;

    while let Some(item) = items.pop() {
      self.inner.allocator.dealloc(item).await;
    }
  }
}

impl<T, A> InnerPool<T, A>
where
  A: ItemAllocator<Output = T>,
{
  async fn release(&self, item: T) {
    if self.permits.is_closed() {
      self.allocator.dealloc(item).await;
    } else {
      let mut items = self.items.lock().await;
      items.push(item);
    }
  }
}
