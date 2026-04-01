// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::Deserialize;

use crate::files::config_files;
use crate::macros::*;
use crate::strings::print_error;

const DL_DISTANCE_MAX_DEFAULT: usize = 5;
const DL_DISTANCE_MIN_DEFAULT: usize = 1;
const DL_DISTANCE_THRESHOLD_DEFAULT: usize = 3;
const DL_DISTANCE_PERCENTAGE_DEFAULT: f64 = std::f64::consts::E * 10.0;

pub static mut DL_DISTANCE_MAX: usize = DL_DISTANCE_MAX_DEFAULT;
pub static mut DL_DISTANCE_MIN: usize = DL_DISTANCE_MIN_DEFAULT;
pub static mut DL_DISTANCE_THRESHOLD: usize = DL_DISTANCE_THRESHOLD_DEFAULT;
pub static mut DL_DISTANCE_PERCENTAGE: f64 = DL_DISTANCE_PERCENTAGE_DEFAULT;

pub fn set_dl_distance_max(max: usize) {
	static_write!(DL_DISTANCE_MAX, max);
}

pub fn set_dl_distance_min(min: usize) {
	static_write!(DL_DISTANCE_MIN, min);
}

pub fn set_dl_distance_threshold(threshold: usize) {
	static_write!(DL_DISTANCE_THRESHOLD, threshold);
}

pub fn set_dl_distance_percentage(percentage: f64) {
	static_write!(DL_DISTANCE_PERCENTAGE, percentage);
}

pub fn get_dl_distance_max() -> usize {
	static_read!(DL_DISTANCE_MAX)
}

pub fn get_dl_distance_min() -> usize {
	static_read!(DL_DISTANCE_MIN)
}

pub fn get_dl_distance_threshold() -> usize {
	static_read!(DL_DISTANCE_THRESHOLD)
}

pub fn get_dl_distance_percentage() -> f64 {
	static_read!(DL_DISTANCE_PERCENTAGE)
}

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct ConfigReader {
	pub dl_distance: Option<DlDistanceConfig>,
}

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct DlDistanceConfig {
	pub max: Option<usize>,
	pub min: Option<usize>,
	pub threshold: Option<usize>,
	pub percentage: Option<f64>,
}

pub struct DLConfig {
	pub max: usize,
	pub min: usize,
	pub threshold: usize,
	pub percentage: f64,
}

impl Default for DLConfig {
	fn default() -> Self {
		Self {
			max: DL_DISTANCE_MAX_DEFAULT,
			min: DL_DISTANCE_MIN_DEFAULT,
			threshold: DL_DISTANCE_THRESHOLD_DEFAULT,
			percentage: DL_DISTANCE_PERCENTAGE_DEFAULT,
		}
	}
}

impl DLConfig {
	pub fn merge(&mut self, reader: ConfigReader) {
		if let Some(reader) = reader.dl_distance {
			merge!(self, reader, max, min, threshold, percentage);
		}
	}

	pub fn apply(&self) {
		set_dl_distance_max(self.max);
		set_dl_distance_min(self.min);
		set_dl_distance_threshold(self.threshold);
		set_dl_distance_percentage(self.percentage);
	}
}

pub fn load_config() {
	let mut dl_config = DLConfig::default();

	for file in config_files() {
		let content = std::fs::read_to_string(&file).expect("Failed to read config file");
		let reader: ConfigReader = toml::from_str(&content).unwrap_or_else(|_| {
			print_error(&format!(
				"Failed to parse config file at {}. Skipping.",
				file
			));
			ConfigReader::default()
		});
		dl_config.merge(reader);
	}
	dl_config.apply();
}
