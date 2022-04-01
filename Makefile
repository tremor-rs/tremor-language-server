publish: 
	cargo publish

test_install: tremor-www-docs
	cargo install --path .
