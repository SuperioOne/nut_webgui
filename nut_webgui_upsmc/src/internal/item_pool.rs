use core::{
  mem::MaybeUninit,
  num::NonZeroUsize,
  ops::{Deref, DerefMut},
};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{AcquireError, Mutex, Semaphore, SemaphorePermit};

pub trait ItemAllocator {
  type Output;
  type Error;

  /// Initializes new item.
  fn init(&self) -> impl Future<Output = Result<Self::Output, Self::Error>>;

  /// Custom async deallocation for item.
  fn dealloc(&self, item: Self::Output) -> impl Future<Output = ()>;

  /// Custom hook to reset or modify an existing pool item before creating a [PoolGuard].
  /// If this function returns false, the pool item will be discarded, and a new one will be dispatched
  /// from either the pool or allocator.
  fn is_valid_state(&self, item: &mut Self::Output) -> impl Future<Output = bool>;
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

#[derive(Debug)]
struct InnerPool<T, A>
where
  A: ItemAllocator<Output = T>,
{
  items: Mutex<VecDeque<T>>,
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
        items: Mutex::new(VecDeque::with_capacity(limit)),
        permits: Semaphore::new(limit),
        allocator,
      }),
    }
  }

  /// Returns an item from pool with provided [ItemAllocator::try_check()] function until a valid
  /// item got found or allocator returns new one.
  pub async fn get_checked(&self) -> Result<PoolGuard<T, A>, ItemPoolError<A::Error>> {
    let permit = self.inner.permits.acquire().await?;
    let mut items_lock = self.inner.items.lock().await;

    loop {
      match items_lock.pop_front() {
        Some(mut item) => {
          if self.inner.allocator.is_valid_state(&mut item).await {
            return Ok(PoolGuard {
              _permit: permit,
              item: MaybeUninit::new(item),
              pool: self.inner.clone(),
            });
          } else {
            _ = self.inner.allocator.dealloc(item).await;
            continue;
          }
        }
        None => {
          return match self.inner.allocator.init().await {
            Ok(item) => Ok(PoolGuard {
              _permit: permit,
              item: MaybeUninit::new(item),
              pool: self.inner.clone(),
            }),
            Err(inner) => Err(ItemPoolError::AllocatorError { inner }),
          };
        }
      };
    }
  }

  /// Returns an item from pool without any check. Returned item might be in an invalid state.
  pub async fn get(&self) -> Result<PoolGuard<T, A>, ItemPoolError<A::Error>> {
    let permit = self.inner.permits.acquire().await?;
    let mut items_lock = self.inner.items.lock().await;

    match items_lock.pop_front() {
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

  pub async fn clear(&mut self) {
    let mut items = self.inner.items.lock().await;

    while let Some(item) = items.pop_back() {
      self.inner.allocator.dealloc(item).await;
    }
  }

  pub async fn close(mut self) {
    self.inner.permits.close();
    Self::clear(&mut self).await;
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
      items.push_back(item);
    }
  }
}
