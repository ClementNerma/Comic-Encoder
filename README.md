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

With `comicenc` being an alias for `cargo run --release --`.

### Compile multiple chapters into volumes of 5 chapters each

```
> comicenc encode --compile 5 -i /home/me/book -o ./build/
```

```
build
├── Volume-1.cbz
├── Volume-2.cbz
└── Volume-3.cbz
```

### Compile chapters into individual volumes

```shell
> comicenc encode --individual -i /home/me/book -o ./build/
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
comicenc encode --single -i /home/me/book -o Book.cbz
```

### Rebuild an existing comic

```shell
comicenc rebuild my-book.pdf
```

This will create a `my-book.cbz` file, a format which is more widely supported by comic readers.

### Options

* `--start-chapter <num>`: ignore every chapter before the provided one (numbers start at 1)
* `--end-chapter <num>`: ignore every chapter after the provided one (numbers start at 1)
* `--dirs-prefix <prefix>`: only consider chapter directories that start by the provided prefix
* `--create-output-dir`: create the output directory if it does not exist yet
* `--extended-images-formats`: allow images with exotic formats, that may not be suppored by the vast majority of comics readers
* `--compress-losslessly`: compress all pictures losslessy - takes quite a bit of time, mostly useless on JPEG/PNG, but useful on BMP images
* `--silent`: do not display anything, except error messages

You can see additional parameters by calling the related subcommand with `--help`.

## Installation

Simply clone the project and run `cargo run --release` inside it, or `cargo build --release` to get a standalone executable.
