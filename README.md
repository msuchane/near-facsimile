# similar-adoc-modules

[![Rust tests](https://github.com/msuchane/similar-adoc-modules/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/msuchane/similar-adoc-modules/actions/workflows/rust-tests.yml)
[![dependency status](https://deps.rs/repo/github/msuchane/similar-adoc-modules/status.svg)](https://deps.rs/repo/github/msuchane/similar-adoc-modules)

Identify modules in Red Hat documentation that are too similar, or identical. Compares text files using the Levenshtein distance metric.

## Usage

1. On Fedora, RHEL, or CentOS, install this program from the Copr repository:

    1. Enable the repository:

        ```
        # dnf copr enable mareksu/similar-adoc-modules
        ```

    2. Install the package:

        ```
        # dnf install similar-adoc-modules
        ```

2. Recursively examine your documentation project:

    ```
    rh-documentation]$ similar-adoc-modules
    ```

3. The program continually prints out file information to the terminal.

    Finally, it saves all statistics sorted by file similarity to the `comparisons.csv` file in the current directory.

## Options

The following options are available:

### Specifying the documentation directory

```
$ similar-adoc-modules <path-to-directory>
```

### Saving the CSV table to a different file

```
$ similar-adoc-modules --csv-path <path-to-new-file>
```

### Setting the lowest reported similarity threshold

```
$ similar-adoc-modules --threshold=<0.8>
```

### Switching to a faster, less accurate comparison

```
$ similar-adoc-modules --fast
```
