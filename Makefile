TOMB_DEBUG_BIN			:=target/debug/tomb
TOMB_RELEASE_BIN		:=target/release/tomb
TOMB_BIN			:=$(TOMB_DEBUG_BIN)
PASSWORD			:="I <3 Nickelback"
PLAINTEXT			:="Hello World"
TOMB_KEY			:= .test-tomb-key.yaml
TOMB_FILE			:= .test-tomb-file.yaml

all: fix release

clean: cls
	@rm -f $(TOMB_FILE) $(TOMB_KEY)
	@rm -fr 0b4sk8d
	@rm -fr *.aes
	@rm -fr *.yaml
	@rm -f *.log
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
	$(TOMB_BIN) genkey -K 1111 -S 2222 -I 3333 -k $(TOMB_KEY) --password $(PASSWORD)
	$(TOMB_BIN) init -k $(TOMB_KEY) -t $(TOMB_FILE)

tomb-save: build cls
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) first-secret "first value"
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) foo bar
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) another-secret "another value"
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) last-secret "last value"
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) "passwords/work/gmail" "Sup@DupAs3cr3T"
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) "passwords/work/vpn" "Sup@1wadsaa"
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) "passwords/work/employee_id" "42069"
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) "passwords/personal/gmail" "s(22;@dup3cr3t"
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) "passwords/personal/spotify" "COCCOp@d99"
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) "passwords/personal/employee_id" "42069"

tomb-list: build cls
	$(TOMB_BIN) list -k $(TOMB_KEY) -t $(TOMB_FILE)

tomb-get: build cls
	$(TOMB_BIN) get -k $(TOMB_KEY) -t $(TOMB_FILE) another-secret

tomb-copy: build cls
	$(TOMB_BIN) copy -k $(TOMB_KEY) -t $(TOMB_FILE) last-secret

tomb-delete: build cls
	$(TOMB_BIN) save -k $(TOMB_KEY) -t $(TOMB_FILE) temporary-secret "some value"
	$(TOMB_BIN) delete -k $(TOMB_KEY) -t $(TOMB_FILE) temporary-secret

tomb-ui: tomb-init tomb-save
	$(TOMB_BIN) ui -T 314 -k $(TOMB_KEY) -t $(TOMB_FILE)

ui:
	cargo run --bin tomb ui -T 400 -k $(TOMB_KEY) -t $(TOMB_FILE)

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
