TOMB_DEBUG_BIN			:=target/debug/tomb
TOMB_RELEASE_BIN		:=target/release/tomb
TOMB_BIN			:=$(TOMB_DEBUG_BIN)
PASSWORD			:="I <3 Nickelback"
PLAINTEXT			:="Hello World"
export TOMB_CONFIG		:= .tomb-config.yaml
export TOMB_KEY			:= .tomb-key.yaml
export TOMB_FILE		:= .tomb-file.yaml
export TOMB_LOG			:= tomb.log

all: fix release

clean: cls
	@rm -f $(TOMB_FILE) $(TOMB_KEY)
	@rm -fr 0b4sk8d
	@rm -fr *.aes
	@rm -fr {.,}*.yaml
	@rm -f {.,}*.log
	@touch {ironpunk,tomb}.log

cls:
	-@reset || tput reset

release: check fix
	@cargo build --release
	cp target/release/tomb ~/usr/bin/

debug: check fix build
	cp target/debug/tomb ~/usr/bin/

fix:
	cargo fix --allow-dirty --allow-staged
	rustfmt --edition 2021 src/*.rs
tmp:
	@rm -rf tmp
	@mkdir -p tmp/{Foo,BAR,BaZ,}/{One,TWO,THree@FouR}
	@for name in $$(find tmp -type d); do uuidgen > $$name/AA; done
	@for name in $$(find tmp -type d); do uuidgen > $$name/bB; done
	@for name in $$(find tmp -type d); do uuidgen > $$name/Cc; done
	@for name in $$(find tmp -type f); do uuidgen > $$name; done

dry-run:tmp
	@cargo run --bin slugify-filenames -- -r tmp --dry-run


build: check
	cargo build

check:
	cargo check --all-targets

silent: tmp cls
	@cargo run --bin slugify-filenames -- -r tmp --silent

test:
	@cargo test

coverage: cls
	grcov . --binary-path target/debug/slugify-filenames -s . -t html --branch --ignore-not-existing -o ./coverage/

tomb: tomb-init tomb-save tomb-list tomb-get tomb-copy

tomb-init: build cls
	$(TOMB_BIN) init -K 1111 -S 2222 -I 3333 --password $(PASSWORD)

tomb-save: build cls
	$(TOMB_BIN) save 'work/gmail' 'Sup@DupAs3cr3T'
	$(TOMB_BIN) save 'work/vpn' 'Sup@1wadsaa'
	$(TOMB_BIN) save 'work/employee_id' '42069'
	$(TOMB_BIN) save '/gmail/my@gmail.com' 's(22;@dup3cr3t'
	$(TOMB_BIN) save '/spotify' 'COCCOp@d99'
	$(TOMB_BIN) save '/netflix' '42069'
	$(TOMB_BIN) save '/github' 'f$$bd^*G0912'
	$(TOMB_BIN) save '/twitter' '**7w337%@$$'

tomb-list: build cls
	$(TOMB_BIN) list

tomb-get: build cls
	$(TOMB_BIN) get /github

tomb-copy: build cls
	$(TOMB_BIN) copy /spotify

tomb-delete: build cls
	$(TOMB_BIN) save temporary-secret "some value"
	$(TOMB_BIN) delete temporary-secret

tomb-ui: clean tomb-init tomb-save
	@gsed 's/cyan/yellow/g' -i $(TOMB_CONFIG)
	$(TOMB_BIN) ui -T 1000

ui:
	cargo run --bin tomb ui -T 500

obfuskat3: cls 0b4sk8d.yaml

0b4sk8d.yaml: $(OBFUSKAT3_BIN)
	$(OBFUSKAT3_BIN) from $(OBFUSKAT3_TARGET_PATH)

unobfuskat3:
	$(OBFUSKAT3_BIN) undo 0b4sk8d.yaml

ipleak: build cls
	$(IPLEAK_BIN)

load: clean build
	./aestest.sh

$(AES256_RELEASE_BIN):
	@cargo build --release

$(AES256_DEBUG_BIN):
	@cargo build

app: clean tomb-ui


.PHONY: all release tmp test dry-run coverage aes256 build check clean test-e2e test-aes-256 test-slugify-filenames bip39 ipleak obfuskat3 app pets fix
