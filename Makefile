# Project settings

## Compiler
ifndef WINDOWS
CC :=			gcc
else
CC :=			x86_64-w64-mingw32-gcc
endif
ifdef RELEASE
ifndef WINDOWS
CFLAGS :=		-O2 -Wall -Werror -Wno-unused-function \
				$(shell pkg-config --cflags gtk+-3.0) \
				$(shell pkg-config --cflags webkit2gtk-4.0) \
				-Iinclude
else
CFLAGS :=		-O2 -Wall -Werror -Wno-unused-function -mwindows \
				-DWIN32 $(shell pkg-config --cflags gtk+-3.0) \
				$(shell pkg-config --cflags webkit2gtk-4.0) \
				-Iinclude
endif
else
CFLAGS :=		-g -Wall -Werror -Wno-unused-function \
				$(shell pkg-config --cflags gtk+-3.0) \
				$(shell pkg-config --cflags webkit2gtk-4.0) \
				-Iinclude
endif
ifndef WINDOWS
LD :=			gcc
else
LD :=			x86_64-w64-mingw32-gc
endif
ifdef RELEASE
ifndef WINDOWS
LDFLAGS :=		$(shell pkg-config --libs gtk+-3.0) \
				$(shell pkg-config --libs webkit2gtk-4.0)
else
LDFLAGS :=		$(shell pkg-config --libs gtk+-3.0) \
				$(shell pkg-config --libs webkit2gtk-4.0)
endif
else
LDFLAGS :=		$(shell pkg-config --libs gtk+-3.0) \
				$(shell pkg-config --libs webkit2gtk-4.0)
endif

## Project

OBJNAME :=		swb
SRC :=			$(wildcard src/*.c)
OBJS =			$(SRC:src/%.c=obj/%.o)
PLUGIN_FLDRS :=	$(shell find plugins -mindepth 1 -maxdepth 1 -type d)
PLUGIN_NAMES :=	$(notdir $(PLUGIN_FLDRS))
ifndef WINDOWS
PLUGINS :=		$(addsuffix .so, $(addprefix lib, $(PLUGIN_NAMES)))
else
PLUGINS :=		$(addsuffix .dll, $(PLUGIN_NAMES))
endif

# Targets

## Helpers

ifdef RELEASE
ifndef WINDOWS
.PHONY: all
all: clean $(OBJNAME) $(PLUGINS)
else
.PHONY: all
all: clean $(OBJNAME).exe $(PLUGINS)
endif
else
.PHONY: all
all: $(OBJNAME) $(PLUGINS)
endif

ifndef WINDOWS
.PHONY: clean
clean:
	$(foreach plugin_fldr,$(PLUGIN_NAMES),make -C plugins/$(plugin_fldr) clean)
	rm -rf obj
	rm -rf *.so
	rm -rf $(OBJNAME)
else
.PHONY: clean
clean:
	$(foreach plugin_fldr,$(PLUGIN_NAMES),make -C plugins/$(plugin_fldr) clean)
	-rm -rf obj
	-rm -rf *.dll
	-rm -rf $(OBJNAME).exe
endif

ifndef WINDOWS
define compile_obj
obj/$(1).o: src/$(1).c $(shell grep -o '#include *[<"][^>"]*\.h[">]' src/$(1).c | sed -E 's/#include *[<"]([^">]+)[">]/\1/' | while read header; do if [ -f include/$header ]; then echo $header; fi; done)
	mkdir -p obj/
	$(CC) -o obj/$(1).o $(CFLAGS) -c src/$(1).c
endef
else
define compile_obj
obj/$(1).o: src\$(1).c $(wildcard include/*)
	-mkdir obj
	$(CC) -o obj\$(1).o $(CFLAGS) -c src\$(1).c
endef
endif

-include $(OBJS:.o=.d)

ifndef WINDOWS
define compile_plugin
lib$(1).so:
	make -C plugins/$(1)
	cp plugins/$(1)/lib$(1).so .
endef
else
define compile_plugin
$(1).dll:
	make -C plugins\$(1)
	cp plugins\$(1)\$(1).dll .
endef
endif

## Main

$(foreach plugin_fldr,$(PLUGIN_NAMES),$(eval $(call compile_plugin,$(plugin_fldr))))

$(foreach file,$(SRC),$(eval $(call compile_obj,$(basename $(notdir $(file))))))

ifndef WINDOWS
$(OBJNAME): $(OBJS)
	$(LD) -o $@ $(OBJS) $(LDFLAGS)
else
$(OBJNAME).exe: $(OBJS)
	$(LD) -o $@ $(OBJS) $(LDFLAGS)
endif
