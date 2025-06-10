<!--
SPDX-FileCopyrightText: Copyright (C) 2025 Chen Linxuan <me@black-desk.cn>

SPDX-License-Identifier: MIT
-->

# clean

[![checks][badge-shields-io-checks]][actions]
[![codecov][badge-shields-io-codecov]][codecov]
[![commit activity][badge-shields-io-commit-activity]][commits]
[![contributors][badge-shields-io-contributors]][contributors]
[![DeepWiki][badge-deepwiki]][deepwiki]

[badge-shields-io-checks]: https://img.shields.io/github/check-runs/black-desk/clean/master
[actions]: https://github.com/black-desk/clean/actions
[badge-shields-io-codecov]: https://codecov.io/gh/black-desk/clean/graph/badge.svg?token=M2XS1G362X
[codecov]: https://codecov.io/github/black-desk/clean
[badge-shields-io-commit-activity]: https://img.shields.io/github/commit-activity/w/black-desk/clean/master
[commits]: https://github.com/black-desk/clean/commits/master
[badge-shields-io-contributors]: https://img.shields.io/github/contributors/black-desk/clean
[contributors]: https://github.com/black-desk/clean/graphs/contributors
[badge-deepwiki]: https://deepwiki.com/badge.svg
[deepwiki]: https://deepwiki.com/black-desk/clean

A simple command-line tool to lint text files for common whitespace and line ending issues.

## Features

- Detects trailing whitespace
- Checks for missing newline at end of file
- Detects CRLF (Windows) line endings
- Detects multiple blank lines at end of file
- Supports output in JSON, YAML, or human-readable format
- Supports custom ignore patterns (`--ignore`)

## Usage

```text
Usage: clean [OPTIONS] [DIR...]

Options:
    --json                  Output results in JSON format
    --yaml                  Output results in YAML format
    --ignore [<PATTERN>...] Ignore file or path (supports glob, can be set multiple times)
    -o, --output <FILE>     Write output to file instead of stdout
    --git [<GIT>]           Only lint files tracked by git (auto-enabled in git repo)
                            If not set, tracked files are linted only if the directory is a git repository. If set to true, only git tracked files are linted. If set to false, all files (not just tracked) are linted, even in a git repository.
                            [possible values: true, false]
    -h, --help              Print help (see a summary with '-h')
    -V, --version           Print version

Arguments:
    [DIR]...                Directories to lint (default: current directory)
                            [default: .]
```

<!--
NOTE: The options and arguments section above should always be kept consistent with the output of `clean --help`.
If you update the CLI, please update both places.
-->

## Example

Lint the current directory and print results in human-readable format:

```sh
clean
```

Lint a specific directory and output as JSON:

```sh
clean --json ./src
```

Ignore files matching a pattern (supports glob, invalid patterns are rejected):

```sh
clean --ignore "*.md" --ignore "target/*"
```

Write output to a file (fails if file is not writable or is a directory):

```sh
clean --output report.txt
```

## Container Image Usage

This project provides an official container image for running `clean` in a fully isolated environment. The image is designed to follow FHS as much as possible:

- The clean binary is located at `/opt/io.github.black-desk/clean/bin/clean`.
- The default working directory is `/mnt`.
- Git is available in the runtime image.

To use the container image, simply mount the directory you want to lint to `/mnt` and pass any arguments as you would to the `clean` binary:

```sh
docker run --rm -v "$(pwd)":/mnt ghrc.io/black-desk/clean:latest --help
```

This will show the help message for the clean tool inside the container. The current directory will be mounted to `/mnt` inside the container, which is also the working directory.

## GitHub Action Usage

This repository provides a reusable GitHub Action for automatically checking text files in your repository using the `clean` tool. You can integrate it into your workflow to detect whitespace and line ending issues on every push or pull request.

### Example Workflow

Create a file at `.github/workflows/lint.yml` in your repository with the following content:

```yaml
name: Lint Text Files

on:
  push:
    paths:
      - '**/*'
  pull_request:
    paths:
      - '**/*'

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Lint with clean
        uses: black-desk/clean@master
        with:
          # Optional: pass arguments to clean
          extra_args: '--ignore "*.md" --ignore "target/*"'
```

#### Inputs

- `extra_args` (optional): Arguments passed as a single string to the clean binary, e.g. `--ignore "*.md" --ignore "target/*"`.

#### Outputs

- `json`: JSON output from the clean tool.
- `yaml`: YAML output from the clean tool.

#### Behavior

- The action runs clean with both `--json` and `--yaml` and writes the results to `${{ steps.<id>.outputs.json }}` and `${{ steps.<id>.outputs.yaml }}`.
- The results are also appended to the GitHub Actions Step Summary.
- You can reference these outputs in subsequent workflow steps.

## License

This project follows [the REUSE Specification](https://reuse.software/spec-3.3/).

You can use [reuse-tool](https://github.com/fsfe/reuse-tool) to generate an SPDX Document of all files in the project like this:

```bash
reuse spdx
```
