//! SmallVec optimizations for OWL2 reasoner
//!
//! This module provides SmallVec-based utilities to reduce heap allocations
//! for small collections commonly used in OWL2 reasoning.

use crate::iri::IRI;
use smallvec::SmallVec;
use std::sync::Arc;

/// Common small sizes for OWL2 collections
pub mod sizes {
    /// Typical size for class expressions in axioms
    pub const CLASS_EXPRESSIONS: usize = 4;

    /// Typical size for property chains
    pub const PROPERTY_CHAINS: usize = 3;

    /// Typical size for annotation collections
    pub const ANNOTATIONS: usize = 8;

    /// Typical size for violation collections
    pub const VIOLATIONS: usize = 16;

    /// Typical size for entity collections in validation
    pub const ENTITIES: usize = 6;
}

/// SmallVec utilities optimized for OWL2 use cases
pub struct SmallVecUtils;

impl SmallVecUtils {
    /// Create a SmallVec for class expressions with optimized inline capacity
    #[inline]
    pub fn class_expressions<T>() -> SmallVec<[T; sizes::CLASS_EXPRESSIONS]> {
        SmallVec::new()
    }

    /// Create a SmallVec for property chains with optimized inline capacity
    #[inline]
    pub fn property_chains<T>() -> SmallVec<[T; sizes::PROPERTY_CHAINS]> {
        SmallVec::new()
    }

    /// Create a SmallVec for annotations with optimized inline capacity
    #[inline]
    pub fn annotations<T>() -> SmallVec<[T; sizes::ANNOTATIONS]> {
        SmallVec::new()
    }

    /// Create a SmallVec for violations with optimized inline capacity
    #[inline]
    pub fn violations<T>() -> SmallVec<[T; sizes::VIOLATIONS]> {
        SmallVec::new()
    }

    /// Create a SmallVec for entities with optimized inline capacity
    #[inline]
    pub fn entities<T>() -> SmallVec<[T; sizes::ENTITIES]> {
        SmallVec::new()
    }

    /// Create a SmallVec for IRIs with optimized inline capacity
    #[inline(always)]
    pub fn iris() -> SmallVec<[Arc<IRI>; sizes::ENTITIES]> {
        SmallVec::new()
    }

    /// Convert a Vec to SmallVec with optimal capacity
    pub fn from_vec<T, const N: usize>(vec: Vec<T>) -> SmallVec<[T; N]> {
        SmallVec::from_vec(vec)
    }

    /// Create a SmallVec from an iterator with size hint
    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<T, I, const N: usize>(iter: I) -> SmallVec<[T; N]>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let mut smallvec = SmallVec::with_capacity(iter.len().min(N));
        smallvec.extend(iter);
        smallvec
    }

    /// Create a pre-allocated SmallVec based on size hint
    pub fn with_capacity<T, const N: usize>(size_hint: usize) -> SmallVec<[T; N]> {
        SmallVec::with_capacity(size_hint.min(N))
    }
}

/// Extension trait for converting between Vec and SmallVec
pub trait SmallVecExt<T> {
    /// Convert to SmallVec with specified inline capacity
    fn to_smallvec<const N: usize>(self) -> SmallVec<[T; N]>;

    /// Convert to SmallVec with optimal inline capacity for this collection type
    fn to_optimized_smallvec(self) -> SmallVec<[T; sizes::ENTITIES]>;
}

impl<T> SmallVecExt<T> for Vec<T> {
    fn to_smallvec<const N: usize>(self) -> SmallVec<[T; N]> {
        SmallVec::from_vec(self)
    }

    fn to_optimized_smallvec(self) -> SmallVec<[T; sizes::ENTITIES]> {
        SmallVec::from_vec(self)
    }
}

/// Extension trait for SmallVec operations specific to OWL2
pub trait OwlSmallVecExt<T> {
    /// Extend with iterator, but only up to the inline capacity to avoid heap allocation
    fn extend_up_to_capacity<I>(&mut self, iter: I) -> usize
    where
        I: IntoIterator<Item = T>,
        Self: Sized;

    /// Check if the SmallVec is using stack allocation (no heap allocation)
    fn is_stack_allocated(&self) -> bool;

    /// Get the inline capacity
    fn inline_capacity(&self) -> usize;
}

impl<T, const N: usize> OwlSmallVecExt<T> for SmallVec<[T; N]> {
    fn extend_up_to_capacity<I>(&mut self, iter: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        let mut count = 0;

        while self.len() < N {
            if let Some(item) = iter.next() {
                self.push(item);
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    #[inline]
    fn is_stack_allocated(&self) -> bool {
        self.len() <= N
    }

    #[inline]
    fn inline_capacity(&self) -> usize {
        N
    }
}

/// Macro for creating SmallVec with common OWL2 patterns
#[macro_export]
macro_rules! smallvec_iris {
    ($($iri:expr),*) => {
        {
            let mut vec = $crate::utils::smallvec::SmallVecUtils::iris();
            $(
                vec.push($iri);
            )*
            vec
        }
    };
}

/// Macro for creating SmallVec with optimized capacity
#[macro_export]
macro_rules! smallvec_with_capacity {
    ($ty:ty, $capacity:expr, $($item:expr),*) => {
        {
            let mut vec = $crate::utils::smallvec::SmallVecUtils::with_capacity::<$ty, $capacity>($capacity);
            $(
                vec.push($item);
            )*
            vec
        }
    };
}
