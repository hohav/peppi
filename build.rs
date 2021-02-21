use std::env;

fn main() {
	let versions = ["v0_2", "v1_0", "v1_2", "v1_3", "v1_4", "v1_5", "v2_0", "v2_1", "v2_2", "v3_0", "v3_2", "v3_5", "v3_6", "v3_7", "v3_8"];

	if let Some(idx) = versions.iter().position(|&v| env::var(format!("CARGO_FEATURE_{}", v.to_uppercase())).is_ok()) {
		for v in versions[..=idx].iter() {
			println!("cargo:rustc-cfg={}", v);
		}
	};

	for v in &versions {
		println!("cargo:rerun-if-env-changed=CARGO_FEATURE_{}", v.to_uppercase());
	}
}
