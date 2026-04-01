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
pub(crate) use {static_read, static_write};

#[macro_export]
macro_rules! remove_env_var {
	($var:expr) => {
		unsafe { std::env::remove_var($var) };
	};
}
