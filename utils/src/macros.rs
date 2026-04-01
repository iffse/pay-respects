// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// used for configuration files
#[macro_export]
macro_rules! merge {
	($self:ident, $reader:ident, $($field:ident),*) => {
		$(
			if let Some($field) = $reader.$field {
				$self.$field = $field;
			}
		)*
	};
}
#[macro_export]
macro_rules! merge_option {
	($self:ident, $reader:ident, $($field:ident),*) => {
		$(
			if let Some($field) = $reader.$field {
				$self.$field = Some($field);
			}
		)*
	};
}

// safe as long as it's not used concurrently
macro_rules! static_read {
	($var:ident) => {
		unsafe { std::ptr::addr_of!($var).read() }
	};
}
macro_rules! static_write {
	($var:ident, $value:expr) => {
		unsafe { std::ptr::addr_of_mut!($var).write($value) }
	};
}
pub(crate) use {merge, static_read, static_write};

#[macro_export]
macro_rules! remove_env_var {
	($var:expr) => {
		unsafe { std::env::remove_var($var) };
	};
}
