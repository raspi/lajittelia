# lajittelia

![GitHub All Releases](https://img.shields.io/github/downloads/raspi/lajittelia/total?style=for-the-badge)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/raspi/lajittelia?style=for-the-badge)
![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/raspi/lajittelia?style=for-the-badge)


Sort files into target directory matching target directory name(s).

Target directory structure example (only directories):

* "Foo"
* "Bar"
* "Quux, Xyzzy"

To be sorted directory (files and directories):

* Hello foo.txt
  * Will be sorted to "Foo" directory
* This is xyzzy.md
  * Will be sorted to "Quux, Xyzzy" directory
* Fooz.ini
  * Will not be sorted because "Fooz" is not "Foo" (word boundary)
* Foo bar.ini
  * Will not be sorted since it matches two directories "Foo" and "Bar"  

## Usage

```
Usage: lajittelia [OPTIONS] --target <TARGET> <PATHS>...

Arguments:
  <PATHS>...  Path(s) to scan for files to be sorted

Options:
  -t, --target <TARGET>  Target directory for sorted files
  -Y, --move-files       Move files? If enabled, files are actually moved
  -h, --help             Print help
  -V, --version          Print version
```

## Example

Sort files in `/mnt/nas/not-sorted` and `/mnt/nas/temp` to `/mnt/nas/sorted`: 

    lajittelia --move-files --target /mnt/nas/sorted /mnt/nas/not-sorted /mnt/nas/temp

