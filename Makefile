build:
	cargo build --release

install:
	cp ./target/release/listitem ~/bin/.

install-stylesheet:
	cp ./resources/listitem.qss ~/bin/.

rcc:
	rcc -binary ./resources/listitem.qrc -o ./resources/listitem.rcc

install-rcc:
	cp ./resources/listitem.rcc ~/bin/. && rm ./resources/listitem.rcc

all: build install install-stylesheet rcc install-rcc