//! This module contains an `EvaluationDomain` abstraction for
//! performing various kinds of polynomial arithmetic on top of
//! fields that are friendly to fast-fourier-transforms (FFTs).
//!
//! A field is FFT-friendly if it contains enough
//! roots of unity to perform the FFT in O(n log n) time.
//! These roots of unity comprise the domain over which
//! polynomial arithmetic is performed.

use ark_ff::FftField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::Rng;
use ark_std::{fmt, hash, vec::Vec};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub mod general;
pub mod mixed_radix;
pub mod radix2;
pub(crate) mod utils;

pub use general::GeneralEvaluationDomain;
pub use mixed_radix::MixedRadixEvaluationDomain;
pub use radix2::Radix2EvaluationDomain;

/// Defines a domain over which finite field (I)FFTs can be performed. The
/// size of the supported FFT depends on the size of the multiplicative
/// subgroup. For efficiency, we recommend that the field has at least one large
/// subgroup generated by a root of unity.
pub trait EvaluationDomain<F: FftField>:
    Copy + Clone + hash::Hash + Eq + PartialEq + fmt::Debug + CanonicalSerialize + CanonicalDeserialize
{
    /// The type of the elements iterator.
    type Elements: Iterator<Item = F> + Sized;

    /// Sample an element that is *not* in the domain.
    fn sample_element_outside_domain<R: Rng>(&self, rng: &mut R, public_rng: bool) -> F {
        let mut t = if public_rng {
            F::pub_rand(rng)
        } else {
            F::rand(rng)
        };
        while self.evaluate_vanishing_polynomial(t).is_zero() {
            t = if public_rng {
                F::pub_rand(rng)
            } else {
                F::rand(rng)
            };
        }
        t
    }

    /// Construct a domain that is large enough for evaluations of a polynomial
    /// having `num_coeffs` coefficients.
    fn new(num_coeffs: usize) -> Option<Self>;

    /// Return the size of a domain that is large enough for evaluations of a
    /// polynomial having `num_coeffs` coefficients.
    fn compute_size_of_domain(num_coeffs: usize) -> Option<usize>;

    /// Return the size of `self`.
    fn size(&self) -> usize;

    /// Return the size of `self` as a field element.
    fn size_as_field_element(&self) -> F {
        F::from(self.size() as u64)
    }

    /// Compute a FFT.
    #[inline]
    fn fft<T: DomainCoeff<F>>(&self, coeffs: &[T]) -> Vec<T> {
        let mut coeffs = coeffs.to_vec();
        self.fft_in_place(&mut coeffs);
        coeffs
    }

    /// Compute a FFT, modifying the vector in place.
    fn fft_in_place<T: DomainCoeff<F>>(&self, coeffs: &mut Vec<T>);

    /// Compute a IFFT.
    #[inline]
    fn ifft<T: DomainCoeff<F>>(&self, evals: &[T]) -> Vec<T> {
        let mut evals = evals.to_vec();
        self.ifft_in_place(&mut evals);
        evals
    }

    /// Compute a IFFT, modifying the vector in place.
    fn ifft_in_place<T: DomainCoeff<F>>(&self, evals: &mut Vec<T>);

    /// Multiply the `i`-th element of `coeffs` with `g^i`.
    fn distribute_powers<T: DomainCoeff<F>>(coeffs: &mut [T], g: F) {
        Self::distribute_powers_and_mul_by_const(coeffs, g, F::one());
    }

    /// Multiply the `i`-th element of `coeffs` with `c*g^i`.
    #[cfg(not(feature = "parallel"))]
    fn distribute_powers_and_mul_by_const<T: DomainCoeff<F>>(coeffs: &mut [T], g: F, c: F) {
        // invariant: pow = c*g^i at the ith iteration of the loop
        let mut pow = c;
        coeffs.iter_mut().for_each(|coeff| {
            *coeff *= pow;
            pow *= &g
        })
    }

    /// Multiply the `i`-th element of `coeffs` with `c*g^i`.
    #[cfg(feature = "parallel")]
    fn distribute_powers_and_mul_by_const<T: DomainCoeff<F>>(coeffs: &mut [T], g: F, c: F) {
        use ark_std::cmp::max;
        let min_parallel_chunk_size = 1024;
        let num_cpus_available = rayon::current_num_threads();
        let num_elem_per_thread = max(coeffs.len() / num_cpus_available, min_parallel_chunk_size);

        ark_std::cfg_chunks_mut!(coeffs, num_elem_per_thread)
            .enumerate()
            .for_each(|(i, chunk)| {
                let offset = c * g.pow([(i * num_elem_per_thread) as u64]);
                let mut pow = offset;
                chunk.iter_mut().for_each(|coeff| {
                    *coeff *= pow;
                    pow *= &g
                })
            });
    }

    /// Compute a FFT over a coset of the domain.
    #[inline]
    fn coset_fft<T: DomainCoeff<F>>(&self, coeffs: &[T]) -> Vec<T> {
        let mut coeffs = coeffs.to_vec();
        self.coset_fft_in_place(&mut coeffs);
        coeffs
    }

    /// Compute a FFT over a coset of the domain, modifying the input vector
    /// in place.
    #[inline]
    fn coset_fft_in_place<T: DomainCoeff<F>>(&self, coeffs: &mut Vec<T>) {
        Self::distribute_powers(coeffs, F::multiplicative_generator());
        self.fft_in_place(coeffs);
    }

    /// Compute a IFFT over a coset of the domain.
    #[inline]
    fn coset_ifft<T: DomainCoeff<F>>(&self, evals: &[T]) -> Vec<T> {
        let mut evals = evals.to_vec();
        self.coset_ifft_in_place(&mut evals);
        evals
    }

    /// Compute a IFFT over a coset of the domain, modifying the input vector in
    /// place.
    #[inline]
    fn coset_ifft_in_place<T: DomainCoeff<F>>(&self, evals: &mut Vec<T>) {
        self.ifft_in_place(evals);
        Self::distribute_powers(evals, F::multiplicative_generator().inverse().unwrap());
    }

    /// Evaluate all the lagrange polynomials defined by this domain at the
    /// point `tau`. This is computed in time O(|domain|).
    /// Then given the evaluations of a degree d polynomial P over this domain,
    /// where d < |domain|, `P(tau)` can be computed as
    /// `P(tau) = sum_{i in [|Domain|]} L_{i, Domain}(tau) * P(g^i)`.
    /// `L_{i, Domain}` is the value of the i-th lagrange coefficient
    /// in the returned vector.
    fn evaluate_all_lagrange_coefficients(&self, tau: F) -> Vec<F>;

    /// Return the sparse vanishing polynomial.
    fn vanishing_polynomial(&self) -> crate::univariate::SparsePolynomial<F>;

    /// This evaluates the vanishing polynomial for this domain at tau.
    fn evaluate_vanishing_polynomial(&self, tau: F) -> F;

    /// Returns the `i`-th element of the domain.
    fn element(&self, i: usize) -> F;

    /// Return an iterator over the elements of the domain.
    fn elements(&self) -> Self::Elements;

    /// The target polynomial is the zero polynomial in our
    /// evaluation domain, so we must perform division over
    /// a coset.
    fn divide_by_vanishing_poly_on_coset_in_place(&self, evals: &mut [F]) {
        let i = self
            .evaluate_vanishing_polynomial(F::multiplicative_generator())
            .inverse()
            .unwrap();

        ark_std::cfg_iter_mut!(evals).for_each(|eval| *eval *= &i);
    }

    /// Given an index which assumes the first elements of this domain are the
    /// elements of another (sub)domain,
    /// this returns the actual index into this domain.
    fn reindex_by_subdomain(&self, other: Self, index: usize) -> usize {
        assert!(self.size() >= other.size());
        // Let this subgroup be G, and the subgroup we're re-indexing by be S.
        // Since its a subgroup, the 0th element of S is at index 0 in G, the first
        // element of S is at index |G|/|S|, the second at 2*|G|/|S|, etc.
        // Thus for an index i that corresponds to S, the index in G is i*|G|/|S|
        let period = self.size() / other.size();
        if index < other.size() {
            index * period
        } else {
            // Let i now be the index of this element in G \ S
            // Let x be the number of elements in G \ S, for every element in S. Then x =
            // (|G|/|S| - 1). At index i in G \ S, the number of elements in S
            // that appear before the index in G to which i corresponds to, is
            // floor(i / x) + 1. The +1 is because index 0 of G is S_0, so the
            // position is offset by at least one. The floor(i / x) term is
            // because after x elements in G \ S, there is one more element from S
            // that will have appeared in G.
            let i = index - other.size();
            let x = period - 1;
            i + (i / x) + 1
        }
    }

    /// Perform O(n) multiplication of two polynomials that are presented by
    /// their evaluations in the domain.
    /// Returns the evaluations of the product over the domain.
    ///
    /// Assumes that the domain is large enough to allow for successful
    /// interpolation after multiplication.
    #[must_use]
    fn mul_polynomials_in_evaluation_domain(&self, self_evals: &[F], other_evals: &[F]) -> Vec<F> {
        assert_eq!(self_evals.len(), other_evals.len());
        let mut result = self_evals.to_vec();
        F::batch_product_in_place(&mut result, other_evals);
        result
    }
}

/// Types that can be FFT-ed must implement this trait.
pub trait DomainCoeff<F: FftField>:
    Copy
    + Send
    + Sync
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + ark_ff::Zero
    + core::ops::MulAssign<F>
{
}

impl<T, F> DomainCoeff<F> for T
where
    F: FftField,
    T: Copy
        + Send
        + Sync
        + core::ops::Add<Output = Self>
        + core::ops::Sub<Output = Self>
        + core::ops::AddAssign
        + core::ops::SubAssign
        + ark_ff::Zero
        + core::ops::MulAssign<F>,
{
}
