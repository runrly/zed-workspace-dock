# Use release-plz and dist for releases

We use release-plz for version bumps, changelog updates, Release PRs, and git tags, and we use cargo-dist for GitHub Release artifacts and generated installers. Merging a Release PR is the human approval step. The post-merge release-plz job creates the version tag, then the tag-triggered cargo-dist workflow builds archives, installers, checksums, attestations, and the GitHub Release.

Release-plz must not create the GitHub Release body because this project treats release artifacts as GitHub Release assets produced by cargo-dist. Homebrew, npm, Scoop, and winget can be added later on top of the same GitHub Release artifacts.
