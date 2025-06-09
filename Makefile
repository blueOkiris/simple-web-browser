# Project settings

## Compiler

CC :=				gcc

ifdef RELEASE
CFLAGS :=			-O2 -Wall -Werror -Wno-unused-function \
					$(shell pkg-config --cflags gtk+-3.0) \
					$(shell pkg-config --cflags webkit2gtk-4.0) \
					-Iinclude
else
CFLAGS :=			-g -Wall -Werror -Wno-unused-function \
					$(shell pkg-config --cflags gtk+-3.0) \
					$(shell pkg-config --cflags webkit2gtk-4.0) \
					-Iinclude
endif
LD :=				gcc
ifdef RELEASE
LDFLAGS :=			$(shell pkg-config --libs gtk+-3.0) \
					$(shell pkg-config --libs webkit2gtk-4.0)
else
LDFLAGS :=			$(shell pkg-config --libs gtk+-3.0) \
					$(shell pkg-config --libs webkit2gtk-4.0)
endif

## Project

OBJNAME :=		swb
SRC :=			$(wildcard src/*.c)
OBJS =			$(SRC:src/%.c=obj/%.o)
PLUGIN_FLDRS :=	$(shell find plugins -mindepth 1 -maxdepth 1 -type d)
PLUGIN_NAMES :=	$(notdir $(PLUGIN_FLDRS))
PLUGINS :=		$(addsuffix .so, $(addprefix lib, $(PLUGIN_NAMES)))

# Targets

## Helpers

ifdef RELEASE
.PHONY: all
all: clean $(OBJNAME) $(PLUGINS)
else
.PHONY: all
all: $(OBJNAME) $(PLUGINS)
endif

.PHONY: clean
clean:
	$(foreach plugin_fldr,$(PLUGIN_NAMES),make -C plugins/$(plugin_fldr) clean;)
	rm -rf obj
	rm -rf *.so
	rm -rf $(OBJNAME)

define compile_obj
obj/$(1).o: src/$(1).c $(shell grep -o '#include *[<"][^>"]*\.h[">]' src/$(1).c | sed -E 's/#include *[<"]([^">]+)[">]/\1/' | while read header; do if [ -f include/$header ]; then echo $header; fi; done)
	mkdir -p obj
	$(CC) -o obj/$(1).o $(CFLAGS) -c src/$(1).c
endef

-include $(OBJS:.o=.d)

define compile_plugin
.PHONY: lib$(1).so
lib$(1).so:
	make -C plugins/$(1)
	cp plugins/$(1)/lib$(1).so .
endef

## Main

$(foreach plugin_fldr,$(PLUGIN_NAMES),$(eval $(call compile_plugin,$(plugin_fldr))))

$(foreach file,$(SRC),$(eval $(call compile_obj,$(basename $(notdir $(file))))))

$(OBJNAME): $(OBJS)
	$(LD) -o $@ $(OBJS) $(LDFLAGS)