BUNDLE_NAME=Tetra Master

all: no

app_bundle: release bundle
clean_app_bundle: clean release clean_bundle bundle

.PHONY: bundle

no:
	@echo "You're probably better off using cargo."

clean:
	cargo clean

release:
	cargo build --release --bin game

clean_bundle:
	rm -rf ./target/$(BUNDLE_NAME).app

bundle:
	mkdir -p ./target/"$(BUNDLE_NAME).app"/Contents/{MacOS,Resources}/
	cp bundle/{Info.plist,PkgInfo} "./target/$(BUNDLE_NAME).app/Contents/"
	cp bundle/Icon.icns "./target/$(BUNDLE_NAME).app/Contents/Resources/"
	cp -r assets "./target/$(BUNDLE_NAME).app/Contents/MacOS/"
	cp target/release/game "./target/$(BUNDLE_NAME).app/Contents/MacOS/TetraMaster"
	strip "./target/$(BUNDLE_NAME).app/Contents/MacOS/TetraMaster"
