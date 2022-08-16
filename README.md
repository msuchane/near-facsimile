# near-facsimile: Find similar or identical text files in a directory

[![Rust tests](https://github.com/msuchane/near-facsimile/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/msuchane/near-facsimile/actions/workflows/rust-tests.yml)
[![dependency status](https://deps.rs/repo/github/msuchane/near-facsimile/status.svg)](https://deps.rs/repo/github/msuchane/near-facsimile)

* **reasonable facsimile**, noun

    : a copy that is not exact but is fairly close

    > The house is a reasonable facsimile of his original home.

    —sometimes used in a joking way

    > I can speak French, or at least a reasonable facsimile of French.

    [“reasonable facsimile,” _Merriam-Webster.com Dictionary_](https://www.merriam-webster.com/dictionary/reasonable%20facsimile)

## Installation

* On Fedora, RHEL, or CentOS, install this program from the Copr repository:

    1. Enable the repository:

        ```
        # dnf copr enable mareksu/near-facsimile
        ```

    2. Install the package:

        ```
        # dnf install near-facsimile
        ```

## Usage

* Recursively examine a directory of text files:

    ```
    dir-with-text]$ near-facsimile
    ```

    The program continually prints out file information to the terminal.

    Finally, it saves all statistics sorted by file similarity to the `comparisons.csv` and `comparisons.json` files in the current directory.

## Options

The following options are available:

### Specifying the documentation directory

```
$ near-facsimile <path-to-directory>
```

### Saving the CSV table to a different file

```
$ near-facsimile --csv-path <path-to-new-file>
```

### Setting the lowest reported similarity threshold

The tools only reports files that are similar over a certain threshold. By default, the threshold is 0.8, or 80% similar.

```
$ near-facsimile --threshold=<0.8>
```

### Switching to a faster, less accurate comparison

By default, the tool uses the _Levenshtein_ metric, which is accurate but rather slow. You can instead compare files using the _Jaro_ metric, which finishes in around half the time, but produces less accurate statistics.

```
$ near-facsimile --fast
```

If you specify the `--fast` option twice (`-ff`), the tool uses the relatively rudimentary but very fast _trigram_ comparison instead:

```
$ near-facsimile --fast --fast
```
