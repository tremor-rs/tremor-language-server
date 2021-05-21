## Release Process

* Bump version in `Cargo.toml`
* Update dependencies (`build-dependencies` and `dev-dependencies`) of `tremor-script` to the new version.
* Checkout the new `tremor-www-docs` release tag in the `tremor-www-docs` submodule folder.
* Test the release by doing `make test_install` and use the language server by writing some trickle/tremor-script.
* Create a PR.
* Checkout `main` and pull the changes from origin.
* Create a git tag and draft a release from it:
  - `git tag -a -m"Release v<VERSION>" v<VERSION>`
  - `git push origin --tag`
* Execute `make publish` from the language server repository root.
* Verify new language server installation via `cargo install tremor-language-server`.
