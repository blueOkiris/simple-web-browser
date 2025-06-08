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
				$(shell pkg-config --cflags gtk+-3.0 webkit2gtk-4.0) \
				-Iinclude
else
CFLAGS :=		-O2 -Wall -Werror -Wno-unused-function -mwindows \
				-DWIN32 $(shell pkg-config --cflags gtk+-3.0) \
				$(shell pkg-config --cflags gtk+-3.0 webkit2gtk-4.0) \
				-Iinclude
endif
else
CFLAGS :=		-g -Wall -Werror -Wno-unused-function \
				$(shell pkg-config --cflags gtk+-3.0) \
				$(shell pkg-config --cflags gtk+-3.0 webkit2gtk-4.0) \
				-Iinclude
endif
ifndef WINDOWS
LD :=			gcc
else
LD :=			x86_64-w64-mingw32-gcc
endif
ifdef RELEASE
ifndef WINDOWS
LDFLAGS :=		-shared
else
LDFLAGS :=		-shared
endif
else
LDFLAGS :=		-shared
endif

## Project

OBJNAME :=		swbbackbtn
SRC :=			$(wildcard src/*.c)
OBJS =			$(SRC:src/%.c=obj/%.o)

# Targets

## Helpers

ifdef RELEASE
ifndef WINDOWS
.PHONY: all
all: clean lib$(OBJNAME).so
else
.PHONY: all
all: clean lib$(OBJNAME).dll
endif
else
.PHONY: all
all: lib$(OBJNAME).so
endif

.PHONY: clean
clean:
	rm -rf obj/
	-rm -rf *.so
	-rm -rf *.dll

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

## Main

$(foreach file,$(SRC),$(eval $(call compile_obj,$(basename $(notdir $(file))))))

ifndef WINDOWS
lib$(OBJNAME).so: $(OBJS)
	$(LD) -o $@ $(LDFLAGS) $(OBJS)
else
$(OBJNAME).dll: $(OBJS)
	$(LD) -o $@ $(LDFLAGS) $(OBJS)
endif
