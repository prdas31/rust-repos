//type_aliases_header.rs
// A comprehensive collection of type aliases to reduce verbosity in Rust projects
// Include this file in your projects to make complex type signatures more readable

use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, VecDeque, LinkedList};
use std::sync::{Arc, Mutex, RwLock, Barrier, Condvar};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::borrow::Cow;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

//------------------------------------------------------------------------------
// Thread-Safety Primitives
//------------------------------------------------------------------------------

// Basic thread-safe wrappers
pub type Shared<T> = Arc<T>;
pub type ThreadSafe<T> = Arc<Mutex<T>>;
pub type ThreadSafeRW<T> = Arc<RwLock<T>>;
pub type AtomicRef<T> = Arc<T>;

// Shorter aliases for synchronization primitives
pub type TSBarrier = Arc<Barrier>;
pub type TSCondvar = Arc<Condvar>;
pub type TSCondPair<T> = Arc<(Mutex<T>, Condvar)>;

// Cell-based thread-safe alternatives
pub type TSCell<T> = Arc<Cell<T>>;
pub type TSRefCell<T> = Arc<RefCell<T>>;

// Common thread-safe collection patterns
pub type TSVec<T> = Arc<Mutex<Vec<T>>>;
pub type TSMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
pub type TSSet<T> = Arc<Mutex<HashSet<T>>>;
pub type TSRWVec<T> = Arc<RwLock<Vec<T>>>;
pub type TSRWMap<K, V> = Arc<RwLock<HashMap<K, V>>>;
pub type TSRWSet<T> = Arc<RwLock<HashSet<T>>>;

// Nested thread-safe collections
pub type NestedTSMap<K1, K2, V> = HashMap<K1, Arc<Mutex<HashMap<K2, V>>>>;
pub type DeepTSMap<K1, K2, K3, V> = HashMap<K1, HashMap<K2, Arc<Mutex<HashMap<K3, V>>>>>;

// Thread-local alternatives (when thread-safety isn't needed)
pub type LocalRef<T> = Rc<T>;
pub type LocalMut<T> = Rc<RefCell<T>>;

//------------------------------------------------------------------------------
// Common Collection Types
//------------------------------------------------------------------------------

// Basic collection aliases
pub type StringMap<V> = HashMap<String, V>;
pub type StringSet = HashSet<String>;
pub type IntMap<V> = HashMap<i64, V>;
pub type IdMap<V> = HashMap<usize, V>;
pub type StrMap<V> = HashMap<&'static str, V>;

// Ordered collections
pub type OrderedMap<K, V> = BTreeMap<K, V>;
pub type OrderedSet<T> = BTreeSet<T>;
pub type Queue<T> = VecDeque<T>;
pub type List<T> = LinkedList<T>;

// Nested collections
pub type MapOfVec<K, V> = HashMap<K, Vec<V>>;
pub type MapOfMap<K1, K2, V> = HashMap<K1, HashMap<K2, V>>;
pub type MapOfSet<K, V> = HashMap<K, HashSet<V>>;

//------------------------------------------------------------------------------
// Error Handling
//------------------------------------------------------------------------------

// Standard result shortcuts
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type StdResult<T, E> = std::result::Result<T, E>;
pub type GenericResult<T> = StdResult<T, BoxError>;
pub type IoResult<T> = std::io::Result<T>;
pub type ParseResult<T> = StdResult<T, std::num::ParseIntError>;
pub type OptResult<T, E> = StdResult<Option<T>, E>;

// For operations that can fail with multiple error types
pub type AnyError = BoxError;
pub type AnyResult<T> = StdResult<T, AnyError>;

//------------------------------------------------------------------------------
// Async Programming
//------------------------------------------------------------------------------

// Common async types
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type AsyncResult<T> = BoxFuture<'static, StdResult<T, AnyError>>;
pub type AsyncFn<T, R> = Box<dyn Fn(T) -> BoxFuture<'static, R> + Send + Sync>;

// Future polling related types
pub type PollResult<T> = Poll<T>;
pub type TaskContext<'a> = Context<'a>;
pub type PollFn<T> = Box<dyn FnMut(&mut TaskContext<'_>) -> Poll<T> + Send>;

// Async collection patterns
pub type AsyncMap<K, V> = Arc<Mutex<HashMap<K, BoxFuture<'static, V>>>>;

// Standard library thread join handle
pub type StdTaskMap<K, T> = Arc<Mutex<HashMap<K, std::thread::JoinHandle<T>>>>;

//------------------------------------------------------------------------------
// Callbacks and Closures
//------------------------------------------------------------------------------

// Common callback types
pub type Callback<T, R> = Box<dyn Fn(T) -> R + Send + Sync>;
pub type SimpleCallback = Box<dyn Fn() + Send + Sync>;
pub type MutCallback<T, R> = Box<dyn FnMut(T) -> R + Send>;
pub type OnceCallback<T, R> = Box<dyn FnOnce(T) -> R + Send>;

// Event handlers
pub type EventHandler<E> = Box<dyn Fn(&E) + Send + Sync>;
pub type EventMap<E> = HashMap<String, Vec<EventHandler<E>>>;
pub type ThreadSafeEventMap<E> = Arc<Mutex<EventMap<E>>>;

//------------------------------------------------------------------------------
// I/O and Resource Handling
//------------------------------------------------------------------------------

// File and path aliases
pub type FilePath = PathBuf;
pub type FileResult<T> = StdResult<T, std::io::Error>;
pub type ReadableFile = Box<dyn Read + Send>;
pub type WritableFile = Box<dyn Write + Send>;

// Resource handling patterns
pub type Resource<T> = Box<dyn AsRef<T>>;
pub type CleanupFn = Box<dyn FnOnce() + Send>;
pub type ResourceMap<K, T> = HashMap<K, (T, CleanupFn)>;

//------------------------------------------------------------------------------
// Common Generic Type Patterns
//------------------------------------------------------------------------------

// For dependency injection
pub type Factory<T> = Box<dyn Fn() -> T + Send + Sync>;
pub type Provider<T> = Arc<dyn Fn() -> T + Send + Sync>;

// For generic traits - using correct trait bounds
// Note: These are designed for use with `dyn Trait` like: DynBox<dyn MyTrait>
pub type DynBox<T> = Box<T>;
pub type DynRef<'a, T> = &'a T;
pub type SendBox<T> = Box<T>;
pub type SyncBox<T> = Box<T>;

// For complex generic type constraints
pub trait Displayable: Display + Debug {}
impl<T> Displayable for T where T: Display + Debug {}
pub type DisplayableType = Box<dyn Displayable>;

//------------------------------------------------------------------------------
// Utility Types for Common Patterns
//------------------------------------------------------------------------------

// For optional ownership scenarios
pub type MaybeOwned<'a, T> = Cow<'a, T>;
pub type OptRef<'a, T> = Option<&'a T>;
pub type OptMut<'a, T> = Option<&'a mut T>;

//------------------------------------------------------------------------------
// Type-Safe Identifiers using PhantomData
//------------------------------------------------------------------------------

/// TypedId uses PhantomData to create type-safe ID wrappers
///
/// # Example
/// ```
/// struct User;
/// struct Product;
///
/// // These are different types at compile time
/// let user_id = TypedId::<User>::new(1);
/// let product_id = TypedId::<Product>::new(1);
///
/// // Won't compile: type mismatch
/// // process_user(product_id);
/// ```
pub struct TypedId<T> {
    id: usize,
    _marker: PhantomData<fn() -> T>, // Use fn() -> T for better variance
}

impl<T> TypedId<T> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> usize {
        self.id
    }
}

// Implement needed traits
impl<T> Clone for TypedId<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for TypedId<T> {}

impl<T> PartialEq for TypedId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for TypedId<T> {}

impl<T> std::hash::Hash for TypedId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Debug for TypedId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypedId<{}>({:?})", std::any::type_name::<T>(), self.id)
    }
}

// Extension of TypedId for common patterns
pub type EntityId<T> = TypedId<T>;
pub type EntityMap<T, V> = HashMap<EntityId<T>, V>;

//------------------------------------------------------------------------------
// Variance Examples with PhantomData
//------------------------------------------------------------------------------

/// Demonstrates covariance - if T is a subtype of U, then Covariant<T> is a subtype of Covariant<U>
pub struct Covariant<T>(PhantomData<T>);

/// Demonstrates contravariance - if T is a subtype of U, then Contravariant<U> is a subtype of Contravariant<T>
pub struct Contravariant<T>(PhantomData<fn(T)>);

/// Demonstrates invariance - no subtype relationship exists between Invariant<T> and Invariant<U>
pub struct Invariant<T>(PhantomData<fn(T) -> T>);

//------------------------------------------------------------------------------
// Advanced Thread Safety for Complex Scenarios
//------------------------------------------------------------------------------

/// A thread-safe reader-writer pattern that simplifies RwLock usage
pub struct ReaderWriterStore<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> ReaderWriterStore<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    pub fn read<F, R>(&self, f: F) -> StdResult<R, std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>>
    where
        F: FnOnce(&T) -> R
    {
        let guard = self.inner.read()?;
        Ok(f(&*guard))
    }

    pub fn write<F, R>(&self, f: F) -> StdResult<R, std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>>
    where
        F: FnOnce(&mut T) -> R
    {
        let mut guard = self.inner.write()?;
        Ok(f(&mut *guard))
    }

    pub fn clone_inner(&self) -> Arc<RwLock<T>> {
        self.inner.clone()
    }
}

impl<T> Clone for ReaderWriterStore<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// A thread-safe wrapper providing atomic operations on a value
pub struct Atomic<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> Atomic<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(data)),
        }
    }

    pub fn with<F, R>(&self, f: F) -> StdResult<R, std::sync::PoisonError<std::sync::MutexGuard<'_, T>>>
    where
        F: FnOnce(&mut T) -> R
    {
        let mut guard = self.inner.lock()?;
        Ok(f(&mut *guard))
    }

    pub fn update<F>(&self, f: F) -> StdResult<(), std::sync::PoisonError<std::sync::MutexGuard<'_, T>>>
    where
        F: FnOnce(&mut T)
    {
        let mut guard = self.inner.lock()?;
        f(&mut *guard);
        Ok(())
    }

    pub fn get_clone(&self) -> StdResult<T, std::sync::PoisonError<std::sync::MutexGuard<'_, T>>>
    where
        T: Clone
    {
        let guard = self.inner.lock()?;
        Ok(guard.clone())
    }
}

impl<T> Clone for Atomic<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

//------------------------------------------------------------------------------
// Example Usage (Commented Out)
//------------------------------------------------------------------------------
/*
// Example 1: Thread-safe user system with minimal verbosity
pub struct UserSystem {
    users: TSRWMap<String, User>,
    sessions: TSMap<EntityId<Session>, UserSession>,
    permissions: NestedTSMap<String, String, Permission>,
    event_handlers: ThreadSafeEventMap<UserEvent>,
    tasks: StdTaskMap<String, ()>,
}

// Example 2: Type-safe IDs
struct User;
struct Product;

fn process_user(id: TypedId<User>) {
    // ...
}

fn main() {
    let user_id = TypedId::<User>::new(1);
    let product_id = TypedId::<Product>::new(1);

    process_user(user_id);
    // process_user(product_id); // Error: types don't match
}

// Example 3: Variance with PhantomData
fn variance_example<'long: 'short, 'short>() {
    // Covariant example
    let _covariant_long: Covariant<&'long str> = Covariant(PhantomData);
    let _covariant_short: Covariant<&'short str> = _covariant_long; // Works with covariance

    // Contravariant example
    let _contra_short: Contravariant<&'short str> = Contravariant(PhantomData);
    let _contra_long: Contravariant<&'long str> = _contra_short; // Works with contravariance

    // Invariant example
    let _invariant_long: Invariant<&'long str> = Invariant(PhantomData);
    // let _invariant_short: Invariant<&'short str> = _invariant_long; // Error with invariance
}
*/