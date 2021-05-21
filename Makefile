publish: tremor-www-docs
	cargo publish

test_install: tremor-www-docs
	cargo package --allow-dirty
	cargo install --path target/package/tremor-language-server-$(shell cargo read-manifest | jq -r .version)
	
tremor-www-docs:
	git submodule update --init
.PHONY: tremor-www-docs
