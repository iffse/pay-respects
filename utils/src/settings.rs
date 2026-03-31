use crate::macros::*;

pub static mut DL_DISTANCE_MAX: usize = 10;
pub static mut DL_DISTANCE_MIN: usize = 2;
pub static mut DL_DISTANCE_PERCENTAGE: usize = 30;

pub fn set_dl_distance_max(max: usize) {
	static_write!(DL_DISTANCE_MAX, max);
}

pub fn set_dl_distance_min(min: usize) {
	static_write!(DL_DISTANCE_MIN, min);
}

pub fn set_dl_distance_percentage(percentage: usize) {
	static_write!(DL_DISTANCE_PERCENTAGE, percentage);
}

pub fn get_dl_distance_max() -> usize {
	static_read!(DL_DISTANCE_MAX)
}

pub fn get_dl_distance_min() -> usize {
	static_read!(DL_DISTANCE_MIN)
}

pub fn get_dl_distance_percentage() -> usize {
	static_read!(DL_DISTANCE_PERCENTAGE)
}
