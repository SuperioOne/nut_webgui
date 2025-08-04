use core::{
  mem::MaybeUninit,
  num::NonZeroUsize,
  ops::{Deref, DerefMut},
  pin::Pin,
};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{AcquireError, Mutex, OwnedSemaphorePermit, Semaphore};

pub enum ItemState<T> {
  Ready(T),
  Destroy(T),
}

pub trait ItemAllocator {
  type Item;
  type Error;

  /// Initializes new item.
  fn init(&self) -> Pin<Box<dyn Future<Output = Result<Self::Item, Self::Error>> + Send + '_>>;

  /// Custom async deallocation for item.
  fn dealloc(&self, item: Self::Item) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

  /// Custom hook to reset or modify an existing pool item before creating a [PoolGuard].
  /// If this function returns false, the pool item will be discarded, and a new one will be dispatched
  /// from either the pool or allocator.
  fn prealloc_check(
    &self,
    item: Self::Item,
  ) -> Pin<Box<dyn Future<Output = ItemState<Self::Item>> + Send + '_>>;
}

#[derive(Debug)]
pub enum ItemPoolError<E> {
  PoolClosed,
  AllocatorError { inner: E },
}

impl<E> core::fmt::Display for ItemPoolError<E>
where
  E: core::error::Error,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      ItemPoolError::PoolClosed => f.write_str("client pool is closed"),
      ItemPoolError::AllocatorError { inner } => core::fmt::Display::fmt(inner, f),
    }
  }
}

impl<E> core::error::Error for ItemPoolError<E> where E: core::error::Error {}

impl<E> From<AcquireError> for ItemPoolError<E> {
  fn from(_: AcquireError) -> Self {
    Self::PoolClosed
  }
}

#[derive(Debug)]
struct InnerPool<A>
where
  A: ItemAllocator,
{
  allocator: A,
  items: Mutex<VecDeque<A::Item>>,
  permits: Arc<Semaphore>,
}

pub struct ItemPool<A>
where
  A: ItemAllocator,
{
  inner: Arc<InnerPool<A>>,
}

pub struct PoolGuard<A>
where
  A: ItemAllocator,
{
  _permit: OwnedSemaphorePermit,
  item: MaybeUninit<A::Item>,
  pool: Arc<InnerPool<A>>,
}

impl<A> Clone for ItemPool<A>
where
  A: ItemAllocator,
{
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<A> PoolGuard<A>
where
  A: ItemAllocator,
{
  pub async fn release(mut self) {
    let item: A::Item = unsafe { self.item.assume_init_read() };
    self.item = MaybeUninit::zeroed();
    self.pool.release(item).await;
  }

  #[inline]
  pub fn into_inner(mut self) -> A::Item {
    let item: A::Item = unsafe { self.item.assume_init_read() };
    self.item = MaybeUninit::zeroed();
    drop(self._permit);

    item
  }
}

impl<A> Deref for PoolGuard<A>
where
  A: ItemAllocator,
{
  type Target = A::Item;

  fn deref(&self) -> &Self::Target {
    unsafe { self.item.assume_init_ref() }
  }
}

impl<A> DerefMut for PoolGuard<A>
where
  A: ItemAllocator,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.item.assume_init_mut() }
  }
}

impl<A> ItemPool<A>
where
  A: ItemAllocator,
{
  pub fn new(limit: NonZeroUsize, allocator: A) -> Self {
    let limit: usize = limit.into();

    Self {
      inner: Arc::new(InnerPool {
        allocator,
        items: Mutex::new(VecDeque::with_capacity(limit)),
        permits: Arc::new(Semaphore::new(limit)),
      }),
    }
  }

  /// Returns an item from pool with provided [ItemAllocator::try_check()] function until a valid
  /// item got found or allocator returns new one.
  pub async fn get_checked(&self) -> Result<PoolGuard<A>, ItemPoolError<A::Error>> {
    let permit = self.inner.permits.clone().acquire_owned().await?;

    loop {
      let item = {
        let mut items_lock = self.inner.items.lock().await;
        items_lock.pop_front()
      };

      match item {
        Some(item) => match self.inner.allocator.prealloc_check(item).await {
          ItemState::Ready(item) => {
            return Ok(PoolGuard {
              _permit: permit,
              item: MaybeUninit::new(item),
              pool: self.inner.clone(),
            });
          }
          ItemState::Destroy(item) => {
            _ = self.inner.allocator.dealloc(item).await;
            continue;
          }
        },
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
  pub async fn get(&self) -> Result<PoolGuard<A>, ItemPoolError<A::Error>> {
    let permit = self.inner.permits.clone().acquire_owned().await?;
    let item = {
      let mut items_lock = self.inner.items.lock().await;
      items_lock.pop_front()
    };

    match item {
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

impl<A> InnerPool<A>
where
  A: ItemAllocator,
{
  async fn release(&self, item: A::Item) {
    if self.permits.is_closed() {
      self.allocator.dealloc(item).await;
    } else {
      let mut items = self.items.lock().await;
      items.push_back(item);
    }
  }
}
