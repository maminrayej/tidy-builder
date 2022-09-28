test: test_stable test_nightly test_doc

test_stable:
	cargo test --test 'stable_*'

test_nightly:
	cargo +nightly test --features better_error --test 'nightly_*'

test_doc:
	cargo test --doc
