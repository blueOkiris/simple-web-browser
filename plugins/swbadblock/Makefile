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
				-Iinclude
else
CFLAGS :=		-g -Wall -Werror -Wno-unused-function -fPIC \
				$(shell pkg-config --cflags gtk+-3.0) \
				$(shell pkg-config --cflags gtk+-3.0 webkit2gtk-4.0) \
				-Iinclude
endif
LD :=			gcc
ifdef RELEASE
LDFLAGS :=		-shared
else
LDFLAGS :=		-shared
endif

## Project

OBJNAME :=		swbadblock
SRC :=			$(wildcard src/*.c)
OBJS =			$(SRC:src/%.c=obj/%.o)

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

define compile_obj
obj/$(1).o: src/$(1).c $(shell grep -o '#include *[<"][^>"]*\.h[">]' src/$(1).c | sed -E 's/#include *[<"]([^">]+)[">]/\1/' | while read header; do if [ -f include/$header ]; then echo $header; fi; done)
	mkdir -p obj/
	$(CC) -o obj/$(1).o $(CFLAGS) -c src/$(1).c
endef

-include $(OBJS:.o=.d)

## Main

$(foreach file,$(SRC),$(eval $(call compile_obj,$(basename $(notdir $(file))))))

lib$(OBJNAME).so: $(OBJS)
	$(LD) -o $@ $(LDFLAGS) $(OBJS)
