# How to Contribute

We'd love to get patches from you!

## Getting Started

You can find [open issues](https://github.com/twitter/rezolus/issues) to work
on labelled 'help-wanted' or 'easy'. If you have and idea for an improvement or
feature that's not covered by an existing issue, please create one first to get
early feedback on your idea.

## Building

A guide to building can be found in the README

## Workflow

We follow the [GitHub Flow Workflow](https://guides.github.com/introduction/flow/)

1.  Fork the project 
1.  Check out the `master` branch 
1.  Create a feature branch
1.  Write code and tests for your change 
1.  From your branch, make a pull request against `twitter/rezolus/master` 
1.  Work with repo maintainers to get your change reviewed 
1.  Wait for your change to be pulled into `twitter/rezolus/master`
1.  Delete your feature branch

## Testing

All testing is driven by the standard Rust toolchain using `cargo test` to run
tests locally. In addition, tests will be run automatically in travis-ci for all
pull requests and merges into this repository.

## Style

We use rustfmt to enforce code style. Please be sure to run `cargo fmt` to make
sure your changes adhere to the style. As rustfmt is under constant development,
you may find that it changes style for files you haven't edited. In this case,
open a [new issue](https://github.com/twitter/rezolus/issues/new). Do not
include formatting changes for unrelated files in your main pull request as it
can make review more time consuming to understand the changes. A separate pull
request to first address any existing style issues will help keep code review
as fast as possible. You can get rustfmt via: `rustup component add rustfmt`

Additionally, we use clippy as our linting tool. Please be sure to run
`cargo clippy` to make sure your changes pass the linter. As with rustfmt,
clippy is under constant development and new lints are added regularly. If you
find that clippy is catching existing issues unrelated to your changes, open a 
[new issue](https://github.com/twitter/rezolus/issues/new). Keeping these
changes in a separate pull request will help keep review as fast as possible.

Style and linting checks will be run automatically in travis-ci for all pull
requests and merges into this repository. 

## Issues

When creating an issue please try to ahere to the following format:

    module-name: One line summary of the issue (less than 72 characters)

    ### Expected behavior

    As concisely as possible, describe the expected behavior.

    ### Actual behavior

    As concisely as possible, describe the observed behavior.

    ### Steps to reproduce the behavior

    List all relevant steps to reproduce the observed behavior.

## Pull Requests

Comments should be formatted to a width no greater than 80 columns.

Files should be exempt of trailing spaces.

We adhere to a specific format for commit messages. Please write your commit
messages along these guidelines. Please keep the line width no greater than 80
columns (You can use `fmt -n -p -w 80` to accomplish this).

    module-name: One line description of your change (less than 72 characters)

    Problem

    Explain the context and why you're making that change.  What is the problem
    you're trying to solve? In some cases there is not a problem and this can be
    thought of being the motivation for your change.

    Solution

    Describe the modifications you've done.

    Result

    What will change as a result of your pull request? Note that sometimes this
    section is unnecessary because it is self-explanatory based on the solution.

Some important notes regarding the summary line:

* Describe what was done; not the result 
* Use the active voice 
* Use the present tense 
* Capitalize properly 
* Do not end in a period — this is a title/subject 
* Prefix the subject with its scope

## Code Review

All pull requests will be reviewed on GitHub and changes may be requested prior
to merging your pull request. Once the changes are approved, the pull request
will be squash-merged into a single commit which retains authorship metadata.

## Documentation

We also welcome improvements to the project documentation or to the existing
docs. Please file an [issue](https://github.com/twitter/rezolus/issues).

# License 

By contributing your code, you agree to license your contribution under the 
terms of the APLv2: https://github.com/twitter/rezolus/blob/master/LICENSE

# Code of Conduct

Read our [Code of Conduct](CODE_OF_CONDUCT.md) for the project.
