# swdist

Simple CLI for string distance and similarity metrics using Rust (`strsim`).

Supports comparing:
- two strings
- two files line-by-line

Includes optional normalization for messy real-world text.

---

## Install

From source:

cargo install --path .

---

## Usage

### String mode

swdist <metric> <string1> <string2> [options]

Examples:

swdist lev kitten sitting  
swdist jw martha marhta  
swdist nlev "Foo, Inc." "foo inc" --norm  

---

### File mode

Compare corresponding lines from two files:

swdist <metric> --files <file1> <file2> [options]

Examples:

swdist lev --files old.txt new.txt  
swdist nlev --files a.txt b.txt --norm  
swdist jw --files left.txt right.txt --no-line-numbers  

Output format (default):

<line_number>\t<result>

Disable line numbers:

--no-line-numbers

If one file is longer, extra lines are compared against an empty string.

---

## Metrics

hamming (equal-length strings only)  
lev, levenshtein  
osa (optimal string alignment)  
damerau, damerau_levenshtein  
jaro  
jw, jaro_winkler  
nlev, normalized_levenshtein  
ndamerau, normalized_damerau_levenshtein  
dice, sorensen_dice  

Notes:
- Hamming returns an error if string lengths differ.
- Jaro/Jaro-Winkler and normalized metrics return similarity scores (0–1).
- Others return integer edit distances.

---

## Normalization

Normalization is optional.

### --norm

Applies a standard cleanup pipeline:

- lowercase
- trim whitespace
- remove accents/diacritics (ASCII fold)
- remove non-alphanumeric characters
- collapse whitespace

Example:

swdist lev "Foo, Inc." "foo inc" --norm  
result: 0  

---

### Individual flags

You can apply steps independently:

--ascii   remove accents (café → cafe)  
--strip   trim leading/trailing whitespace  
--alnum   remove non-alphanumeric characters  
--space   collapse whitespace  

Example:

swdist lev "café au lait" "cafeaulait" --ascii --alnum  

---

## Notes

- `--norm` overrides individual normalization flags.
- In `--norm`, whitespace collapsing has little effect because non-alphanumeric characters are removed first.
- Performance depends on metric:
  - Hamming is O(n)
  - Levenshtein and related metrics are O(n²)

---

## Typical use cases

Compare columns extracted from CSVs:

xsv select col1 file1.csv > a.txt  
xsv select col1 file2.csv > b.txt  

swdist lev --files a.txt b.txt --norm  

Combine with process substitution:

swdist lev --files <(cmd1) <(cmd2)

---

## License

MIT or Apache-2.0
