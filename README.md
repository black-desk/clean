<!--
SPDX-License-Identifier: AGPL-3.0-or-later
SPDX-FileCopyrightText: Copyright (C) 2025 Chen Linxuan <me@black-desk.cn>
-->

# clean

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

This repository provides a reusable GitHub Action for linting text files in your repository using `clean`. You can integrate it into your workflow to automatically check for whitespace and line ending issues on every push or pull request.

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
          # args: --ignore "*.md" --ignore "target/*"
```

#### Inputs

- `args` (optional): Arguments to pass to the `clean` CLI. For example: `--ignore "*.md"`.

#### Output

The action will run `clean` on your repository and fail the workflow if any issues are found. You can customize the arguments as needed.

## License

This project is licensed under the GNU Affero General Public License v3.0. See the LICENSE file for details.
