use std::{
	collections::BTreeSet,
	ops::{Deref, DerefMut},
	rc::Rc,
};

/// Tracks the virtual files that the program used (they overwrite something)
#[derive(Default, Debug)]
pub(crate) struct VirtualFileOverrideList(BTreeSet<Rc<str>>);

impl Deref for VirtualFileOverrideList {
	type Target = BTreeSet<Rc<str>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for VirtualFileOverrideList {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
