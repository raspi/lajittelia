# lajittelia

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
