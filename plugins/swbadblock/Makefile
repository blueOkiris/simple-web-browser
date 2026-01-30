# Project settings

## Windows Cross-Compilation

MXE_COMMIT :=	f485e7f11d3383c5891d3ec73c56360fcf3aa124
MXE_ZIP :=		$(MXE_COMMIT).zip
MXE_FLDR :=		mxe-$(MXE_COMMIT)
MXE_TARGET :=	x86_64-w64-mingw32.static

## Compiler

CC :=			gcc
ifdef RELEASE
CFLAGS :=		-O2 -Wall -Werror -Wno-unused-function -fPIC \
				$(shell pkg-config --cflags gtk+-3.0) \
				$(shell pkg-config --cflags gtk+-3.0 webkit2gtk-4.0) \
				$(shell pkg-config --cflags json-c) \
				-Iinclude
else
CFLAGS :=		-g -Wall -Werror -Wno-unused-function -fPIC \
				$(shell pkg-config --cflags gtk+-3.0) \
				$(shell pkg-config --cflags gtk+-3.0 webkit2gtk-4.0) \
				$(shell pkg-config --cflags json-c) \
				-Iinclude
endif
LD :=			gcc
ifdef RELEASE
LDFLAGS :=		-shared $(shell pkg-config --libs json-c) -lcurl
else
LDFLAGS :=		-shared $(shell pkg-config --libs json-c) -lcurl
endif

## Project

OBJNAME :=		swbadblock
SRC :=			$(wildcard src/*.c)
OBJS =			$(SRC:src/%.c=obj/%.o)

## SafariConverter

SCL_VERS :=		4.1.0
SCL_URL := 		https://github.com/AdguardTeam/SafariConverterLib/archive/refs/tags/v$(SCL_VERS).tar.gz
SCL_DIR := 		adguard2webkit/SafariConverterLib-$(SCL_VERS)
ifdef RELEASE
AG2WK_LIB := 	adguard2webkit/.build/release/libAdGuard2WebKit.a
SCL_LIB :=		adguard2webkit/.build/release/libContentBlockerConverter.a
SWIFTFLAGS := 	-c release
else
AG2WK_LIB := 	adguard2webkit/.build/debug/libAdGuard2WebKit.a
SCL_LIB :=		adguard2webkit/.build/debug/libContentBlockerConverter.a
SWIFTFLAGS := 	-c debug
endif

# Targets

## Helpers

ifdef RELEASE
.PHONY: all
all: clean lib$(OBJNAME).so
else
.PHONY: all
all: lib$(OBJNAME).so
endif

.PHONY: clean
clean:
	rm -rf obj/
	-rm -rf *.so
	-rm -rf *.dll
	rm -rf $(SCL_DIR)
	rm -rf adguard2webkit/v$(SCL_VERS).tar.gz
	rm -rf adguard2webkit/.build

define compile_obj
obj/$(1).o: src/$(1).c $(shell grep -o '#include *[<"][^>"]*\.h[">]' src/$(1).c | sed -E 's/#include *[<"]([^">]+)[">]/\1/' | while read header; do if [ -f include/$header ]; then echo $header; fi; done)
	mkdir -p obj/
	$(CC) -o obj/$(1).o $(CFLAGS) -c src/$(1).c
endef

-include $(OBJS:.o=.d)

## SafariConverter Library

adguard2webkit/v$(SCL_VERS).tar.gz:
	cd adguard2webkit && wget -O v$(SCL_VERS).tar.gz $(SCL_URL)

$(SCL_DIR)/Package.swift: adguard2webkit/v$(SCL_VERS).tar.gz
	cd adguard2webkit && tar -xzf v$(SCL_VERS).tar.gz
	cd $(SCL_DIR) && patch -p0 < ../scl-package-reduction.patch

$(AG2WK_LIB): $(SCL_DIR)/Package.swift $(wildcard adguard2webkit/Sources/*.swift)
	cd adguard2webkit && swift build $(SWIFTFLAGS)

## Main

$(foreach file,$(SRC),$(eval $(call compile_obj,$(basename $(notdir $(file))))))

lib$(OBJNAME).so: $(OBJS) $(AG2WK_LIB)
	$(LD) -o $@ $(LDFLAGS) $(OBJS) $(AG2WK_LIB) $(SCL_LIB)
