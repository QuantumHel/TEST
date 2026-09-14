pub trait Edge {
	fn weight(&self) -> f64;

	fn nodes(&self) -> Vec<usize>;
}

pub use encapsulation::ConstCardinalityEdge;

/// Allows for the trait and type to have the same name
mod encapsulation {
	use super::Edge;

	/// A marker trait that promises that a type implements [Edge] in such a way
	/// that [Edge::nodes] can only return vectors with the size indicated by
	/// [ConstCardinalityEdge::CARDINALITY].
	///
	/// This means that users can assume that all [Edge]s of this type the indicated
	/// cardinality of [ConstCardinalityEdge::CARDINALITY].
	pub trait ConstCardinalityEdge: Edge {
		/// The constant cardinality of the [Edge].
		///
		/// Because `min_generic_const_args` is not stable this requires the use of
		/// a helper struct [Cardinality\<N\>](Cardinality) where `N` indicates the
		/// cardinality.  
		#[allow(private_bounds)]
		type CARDINALITY: Cardinality;
	}

	/// This is a trait for evading missing constant handling that is behind
	/// `min_generic_const_args`. The only struct implementing this trait is
	/// [Cardinality\<N\>](Cardinality) which contains the needed constant `N`.
	///
	/// The trait is intentionally hidden.
	#[doc(hidden)]
	pub(super) trait Cardinality {}
}

/// This is a helper struct to indicate the cardinality `N` within
/// [ConstCardinalityEdge].
///
/// This struct is for going around the missing const handling from
/// `min_generic_const_args`.
pub struct Cardinality<const N: usize>;

impl<const N: usize> encapsulation::Cardinality for Cardinality<N> {}
