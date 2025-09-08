# Powerscan

Powerscan aims to become a more powerful, cross-platform alternative to programs like [GNOME Simple Scan](https://apps.gnome.org/SimpleScan/).
The Powerscan GUI is built with GTK4 and supports the following backends on their respective platforms:

- [SANE](https://www.sane-project.org/) on Linux

## Building & Testing

### Linux
The Linux backend requires both sane and gtk4 development libraries to be installed on the system.
The device used for unit tests can be specified with the `POWERSCAN_SANE_TEST_DEVICE` environment variable.
If left blank, the `test:0` device will be used instead.
For more information on `test:` devices, see the [`sane-test` manpage](http://www.sane-project.org/man/sane-test.5.html).

## Roadmap

- [X] Basic SANE Backend on Linux
- [ ] Initial GUI to scan and display a page
- [ ] Initial support for TWAIN on Windows
- [ ] More powerful GUI implementation
- [ ] MacOS Support

## Contributing

Currently, Powerscan is in very early stages of development.
All contributions are welcome.
If anyone with a Mac wants to commit to writing the MacOS backend, help is very much appreciated.
