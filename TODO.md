# TODO

## High prio
- facilitate cargo install
    - README.md
- wdgrep: make sure only known dump filenames/prefixes can be searched

### wdget:
- color?
- new options --bunzip2-binary, --bunzip2-options
- gz, 7z decompression support
- arg checking (disallow empty as well?)
- support updating from incr(emental) dumps

### wdgrep
- bunzip2 support via lib
- ci, more tests, coverage
- set up benchmarking
- README.md
- filter by title: --intitle
- kill decompress process on exit
- remove unintuitive dump prefix arg syntax
- only show revision in multi-revision search

## Long-term
- refactor: main calls fn returning Result<()> for full control over dropping and exit code
- benchmark mimalloc instead of snmalloc (much more widely used)
- search progress display
- parse siteinfo and allow passing namespaces by name?
- print match statistics (how many matches in how many articles, percentage of pages matching, ...)
- output formats: normal, csv, json, wikitext
- only print matches
- for text output: one-line-per-match as an option
- benchmark: use less processes with 7z?
- benchmark (Windows): disable 7z multi-threading?
- print captured groups, maybe also s/../../
- kib/mib/gib, hh:mm:ss
- clap_generate
- use (color-)eyre instead of anyhow for backtraces
- support multiple regex engines/regex engine switching
- parquet/duckdb export

### Full dump only improvements
- only useful for full dump: filter by user, comment, minor, timestamp (between, before, after, --as-of)
- search added/removed text?


## A man can dream...
- Aarch64 Neon memchr implementation
- non-copying XML parser
- SIMD UTF-8

## Abandonded ideas
- show performance statistics on break too
- make use of index and parallelize single-file bzip2 extraction by using multi-streams (abandoned: bzip2 too slow in any case, no need to waste time on it)
- make use of index when bzip2 searching with --intitle (abandoned: bzip2 too slow in any case, no need to waste time on it)
- wdget: --resume-partial/--keep-partial
- wdget: automatically try again if intermittent network issue (or w/ --retry)?

## wdget
- progress: show ETA?
- download --verify or --no-verify option?
- colorize
- support decompressing  .gz/.xz while dl'ing as well?
- --overwrite (or --force ?)
- tests?

## further ideas:
- dump update into wdget or wdupdate
- wdcat, wdls
- better error handling