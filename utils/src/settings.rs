// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::Deserialize;

use crate::files::config_files;
use crate::macros::*;
use crate::strings::print_error;

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum SearchType {
	DamerauLevenshtein,
	TrigramDamerauLevenshtein,
}
use SearchType::*;

const SEARCH_TYPE_DEFAULT: SearchType = TrigramDamerauLevenshtein;
const SEARCH_THRESHOLD_DEFAULT: usize = 3;

const TRIGRAM_MINIMUM_SCORE_DEFAULT: f32 = std::f32::consts::E * 0.1;

const DL_DISTANCE_MAX_DEFAULT: usize = 5;
const DL_DISTANCE_MIN_DEFAULT: usize = 1;
const DL_DISTANCE_PERCENTAGE_DEFAULT: f32 = std::f32::consts::E * 10.0;

pub static mut SEARCH_TYPE: SearchType = SEARCH_TYPE_DEFAULT;
pub static mut SEARCH_THRESHOLD: usize = SEARCH_THRESHOLD_DEFAULT;

pub static mut TRIGRAM_MINIMUM_SCORE: f32 = TRIGRAM_MINIMUM_SCORE_DEFAULT;

pub static mut DL_DISTANCE_MAX: usize = DL_DISTANCE_MAX_DEFAULT;
pub static mut DL_DISTANCE_MIN: usize = DL_DISTANCE_MIN_DEFAULT;
pub static mut DL_DISTANCE_PERCENTAGE: f32 = DL_DISTANCE_PERCENTAGE_DEFAULT;

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct ConfigReader {
	pub search_type: Option<SearchType>,
	pub search_threshold: Option<usize>,
	pub trigram: Option<TrigramConfigReader>,
	pub dl_distance: Option<DlConfigReader>,
}

#[derive(Deserialize, Default)]
pub struct DlConfigReader {
	pub max: Option<usize>,
	pub min: Option<usize>,
	pub percentage: Option<f32>,
}

#[derive(Deserialize, Default)]
pub struct TrigramConfigReader {
	pub minimum_score: Option<f32>,
}

pub struct DLConfig {
	pub max: usize,
	pub min: usize,
	pub percentage: f32,
}

pub struct TrigramConfig {
	pub minimum_score: f32,
}

pub struct Config {
	pub search_type: SearchType,
	pub search_threshold: usize,
	pub dl_distance: DLConfig,
	pub trigram: TrigramConfig,
}

impl Default for DLConfig {
	fn default() -> Self {
		Self {
			max: DL_DISTANCE_MAX_DEFAULT,
			min: DL_DISTANCE_MIN_DEFAULT,
			percentage: DL_DISTANCE_PERCENTAGE_DEFAULT,
		}
	}
}

impl Default for TrigramConfig {
	fn default() -> Self {
		Self {
			minimum_score: TRIGRAM_MINIMUM_SCORE_DEFAULT,
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			search_type: SEARCH_TYPE_DEFAULT,
			search_threshold: SEARCH_THRESHOLD_DEFAULT,
			dl_distance: DLConfig::default(),
			trigram: TrigramConfig::default(),
		}
	}
}

impl Config {
	pub fn merge(&mut self, reader: ConfigReader) {
		merge!(self, reader, search_type, search_threshold);
		if let Some(reader) = reader.dl_distance {
			let dl_distance = &mut self.dl_distance;
			merge!(dl_distance, reader, max, min, percentage);
		}
		if let Some(reader) = reader.trigram {
			let trigram = &mut self.trigram;
			merge!(trigram, reader, minimum_score);
		}
	}

	pub fn apply(&self) {
		set_search_type(self.search_type);
		set_search_threshold(self.search_threshold);
		set_trigram_minimum_score(self.trigram.minimum_score);
		set_dl_distance_max(self.dl_distance.max);
		set_dl_distance_min(self.dl_distance.min);
		set_dl_distance_percentage(self.dl_distance.percentage);
	}
}

// impl DLConfig {
// 	pub fn merge(&mut self, reader: ConfigReader) {
// 		if let Some(reader) = reader.dl_distance {
// 			merge!(self, reader, max, min, percentage);
// 		}
// 	}

// 	pub fn apply(&self) {
// 		set_dl_distance_max(self.max);
// 		set_dl_distance_min(self.min);
// 		set_dl_distance_percentage(self.percentage);
// 	}
// }

pub fn load_config() {
	let mut config = Config::default();

	for file in config_files() {
		let content = std::fs::read_to_string(&file).expect("Failed to read config file");
		let reader: ConfigReader = toml::from_str(&content).unwrap_or_else(|_| {
			print_error(&format!(
				"Failed to parse config file at {}. Skipping.",
				file
			));
			ConfigReader::default()
		});
		config.merge(reader);
	}
	config.apply();
}

pub fn set_search_type(search_type: SearchType) {
	static_write!(SEARCH_TYPE, search_type);
}

pub fn set_search_threshold(threshold: usize) {
	static_write!(SEARCH_THRESHOLD, threshold);
}

pub fn set_trigram_minimum_score(minimum_score: f32) {
	static_write!(TRIGRAM_MINIMUM_SCORE, minimum_score);
}

pub fn set_dl_distance_max(max: usize) {
	static_write!(DL_DISTANCE_MAX, max);
}

pub fn set_dl_distance_min(min: usize) {
	static_write!(DL_DISTANCE_MIN, min);
}

pub fn set_dl_distance_percentage(percentage: f32) {
	static_write!(DL_DISTANCE_PERCENTAGE, percentage);
}

pub fn get_search_type() -> SearchType {
	static_read!(SEARCH_TYPE)
}

pub fn get_search_threshold() -> usize {
	static_read!(SEARCH_THRESHOLD)
}

pub fn get_trigram_minimum_score() -> f32 {
	static_read!(TRIGRAM_MINIMUM_SCORE)
}

pub fn get_dl_distance_max() -> usize {
	static_read!(DL_DISTANCE_MAX)
}

pub fn get_dl_distance_min() -> usize {
	static_read!(DL_DISTANCE_MIN)
}

pub fn get_dl_distance_percentage() -> f32 {
	static_read!(DL_DISTANCE_PERCENTAGE)
}
