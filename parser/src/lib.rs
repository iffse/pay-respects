// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::Path;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn parse_rules(input: TokenStream) -> TokenStream {
	let directory = input.to_string().trim_matches('"').to_string();
	let rules = pay_respects_rule_engine::parse_dir_with(
		Path::new(&directory),
		pay_respects_rule_engine::ParseOptions {
			allow_scripts: false,
		},
	)
	.unwrap_or_else(|err| panic!("failed to parse rules: {}", err));
	let bytes = postcard::to_allocvec(&rules).expect("failed to encode rules");
	let bytes = bytes.iter();

	quote! {
		::pay_respects_rule_engine::decode(&[#(#bytes),*])
			.expect("failed to decode compiled rules")
	}
	.into()
}
