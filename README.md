# Comic Encoder

Comic Encoder is a command-line tool that enables compilation and extraction of comic archives.

## Features

Main features are:

* Compiling groups of chapters into several volumes (e.g. you have dozens of chapters and want to make volumes of 10 chapters)
* Compiling chapters into individual volumes (e.g. you want one volume per chapter, or you simply want to compile multiple comics at once)
* Compiling groups of chapters into a single volume (e.g. you have all chapters of a book and want to get a single archive out of it)
* Rebuild comics (e.g. convert a PDF comic to a CBZ one, to use a more widely supported format)
* Uses [natural sorting algorithm](lib/natsort.rs) to determine chapters and pages order

Supported formats are `.zip` / `.cbz` and `.pdf` files.
Support is planned for `.rar` / `.cbr` and `.7z` / `.cb7` files.

## Usage

Considering the following directory at `/home/me/book`:

```
/home/me/book
├── FirstChapter_1
├── MyChapter_10
├── MyChapter_11
├── MyChapter_2
├── MyChapter_3
├── MyChapter_4
├── MyChapter_5
├── MyChapter_6
├── MyChapter_7
├── MyChapter_8
├── MyChapter_9
└── ZChapter_12
```

### Compile multiple chapters into volumes of 5 chapters each

```
> comic-enc encode /home/me/book compile 5 -o ./build/
```

```
build
├── Volume-1.cbz
├── Volume-2.cbz
└── Volume-3.cbz
```

### Compile chapters into individual volumes

```shell
> comic-enc encode /home/me/book each -o ./build/
```

```
build
├── FirstChapter_1.cbz
├── MyChapter_10.cbz
├── MyChapter_11.cbz
├── MyChapter_2.cbz
├── MyChapter_3.cbz
├── MyChapter_4.cbz
├── MyChapter_5.cbz
├── MyChapter_6.cbz
├── MyChapter_7.cbz
├── MyChapter_8.cbz
├── MyChapter_9.cbz
└── ZChapter_12.cbz
```

### Compile multiple chapters into a single volume

```shell
comic-enc encode /home/me/book single
```

This will create a file named `book.cbz` containing all chapters.

### Compile a single directory of pictures

```shell
comic-enc encode /home/me/pictures single
```

This will create a single file named `pictures.cbz` containing all pictures directly under `/home/me/pictures`.

### Extract an existing comic

```shell
comic-enc decode ./build/FirstChapter_1.cbz
```

This will create a directory named `FirstChapter_1` containing the volume's images.

### Options

You can see additional parameters by calling the related subcommand with `--help`.

## Installation

Simply clone the project and run `cargo install --path .` inside it.
