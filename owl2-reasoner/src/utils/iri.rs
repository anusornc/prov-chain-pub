//! Performance-optimized IRI utilities
//!
//! This module provides optimized utilities for working with IRIs,
//! focusing on reducing `Arc<IRI>` cloning operations and improving memory efficiency.

use crate::iri::IRI;
use std::sync::Arc;

/// Optimized IRI iterator adapter for map operations
///
/// This provides a zero-cost way to chain IRI operations
/// with minimal intermediate allocations.
pub struct IriIterator<'a, I> {
    inner: I,
    _phantom: std::marker::PhantomData<&'a Arc<IRI>>,
}

/// Optimized IRI collection utilities
pub struct IriUtils;

impl IriUtils {
    /// Collect IRIs from entity references with minimal cloning
    ///
    /// This is optimized for cases where you have a collection of entities
    /// and need to extract their IRIs with minimal memory overhead.
    pub fn collect_iris_from_entities<'a, I, E>(entities: I) -> Vec<Arc<IRI>>
    where
        I: IntoIterator<Item = &'a E>,
        E: crate::entities::Entity + 'a,
    {
        entities.into_iter().map(|e| Arc::clone(e.iri())).collect()
    }

    /// Convert `Arc<IRI>` collection to IRI collection with optimized cloning
    ///
    /// Uses Arc::into_inner when possible to avoid double cloning,
    /// falls back to standard cloning for shared references.
    pub fn arc_iris_to_iris(arc_iris: Vec<Arc<IRI>>) -> Vec<IRI> {
        arc_iris
            .into_iter()
            .map(|iri| Arc::try_unwrap(iri).unwrap_or_else(|iri| (*iri).clone()))
            .collect()
    }

    /// Create a `Vec<IRI>` from entity references with single cloning step
    pub fn entities_to_iris<'a, I, E>(entities: I) -> Vec<IRI>
    where
        I: IntoIterator<Item = &'a E>,
        E: crate::entities::Entity + 'a,
    {
        entities
            .into_iter()
            .map(|e| {
                Arc::into_inner(Arc::clone(e.iri())).unwrap_or_else(|| (*e.iri()).clone().into())
            })
            .collect()
    }

    /// Create an optimized IRI iterator from entity references
    pub fn iter_iris_from_entities<'a, I, E>(entities: I) -> IriIterator<'a, I::IntoIter>
    where
        I: IntoIterator<Item = &'a E>,
        E: crate::entities::Entity + 'a,
    {
        IriIterator {
            inner: entities.into_iter(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Pre-allocate a `Vec` with reasonable capacity for IRI collections
    ///
    /// Helps reduce reallocations during IRI collection operations.
    pub fn preallocate_iri_vec(size_hint: usize) -> Vec<Arc<IRI>> {
        Vec::with_capacity(size_hint.max(8)) // Minimum capacity to avoid tiny allocations
    }

    /// Clone `Arc<IRI>` using the most efficient method available
    ///
    /// This is a convenience function that ensures optimal Arc cloning.
    pub fn clone_arc_iri(iri: &Arc<IRI>) -> Arc<IRI> {
        Arc::clone(iri)
    }

    /// Batch convert IRIs with capacity pre-allocation
    pub fn batch_convert_iris(arc_iris: &[Arc<IRI>]) -> Vec<IRI> {
        let mut result = Vec::with_capacity(arc_iris.len());
        result.extend(arc_iris.iter().map(|iri| {
            Arc::into_inner(Arc::clone(iri)).unwrap_or_else(|| Arc::clone(iri).clone().into())
        }));
        result
    }
}

impl<'a, I, E> Iterator for IriIterator<'a, I>
where
    I: Iterator<Item = &'a E>,
    E: crate::entities::Entity + 'a,
{
    type Item = Arc<IRI>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|e| Arc::clone(e.iri()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Extension trait for collections of entities to provide optimized IRI operations
pub trait EntityIriExt {
    /// Extract IRIs from entity collection with minimal cloning
    fn extract_iris_optimized(&self) -> Vec<Arc<IRI>>;
}

impl<E: crate::entities::Entity> EntityIriExt for Vec<E> {
    fn extract_iris_optimized(&self) -> Vec<Arc<IRI>> {
        IriUtils::collect_iris_from_entities(self)
    }
}

impl<E: crate::entities::Entity> EntityIriExt for &[E] {
    fn extract_iris_optimized(&self) -> Vec<Arc<IRI>> {
        IriUtils::collect_iris_from_entities(*self)
    }
}

/// Macro for optimized IRI collection in performance-critical code
#[macro_export]
macro_rules! collect_iris {
    ($($entity:expr),*) => {
        {
            let mut iris = $crate::utils::iri::IriUtils::preallocate_iri_vec(count_args!($($entity),*));
            $(
                iris.push($crate::utils::iri::IriUtils::clone_arc_iri($entity.iri()));
            )*
            iris
        }
    };
}

/// Helper macro for counting arguments (used by collect_iris)
#[macro_export]
macro_rules! count_args {
    () => { 0 };
    ($x:expr $(, $xs:expr)*) => { 1 + count_args!($($xs),*) };
}
