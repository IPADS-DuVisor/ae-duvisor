ifeq ($(strip $(V)),)
	ifeq ($(findstring s,$(filter-out --%,$(MAKEFLAGS))),)
		E = @echo
	else
		E = @\#
	endif
	Q = @
else
	E = @\#
	Q =
endif
export E Q

CROSS_COMPILE 	:= riscv64-unknown-linux-gnu-

CC 	:= $(CROSS_COMPILE)gcc
CFLAGS 	:= -fPIC
LD 	:= $(CROSS_COMPILE)ld
LDFLAGS :=
OBJCOPY := $(CROSS_COMPILE)objcopy

PROGRAM	:= lkvm.o

OBJS 	+= disk/core.o
OBJS 	+= disk/raw.o
OBJS 	+= blk.o
OBJS 	+= net.o
OBJS 	+= mmio.o
OBJS 	+= core.o
OBJS	+= util/init.o
OBJS    += util/iovec.o
OBJS	+= util/rbtree.o
OBJS	+= util/threadpool.o
OBJS	+= util/parse-options.o
OBJS	+= util/rbtree-interval.o
OBJS	+= util/strbuf.o
OBJS	+= util/read-write.o
OBJS	+= util/util.o

DEFINDES 	+= -DCONFIG_RISCV
ARCH_INCLUDE 	:= riscv/include
CFLGAS 		+= -mabi=lp64d -march=rv64gc

LIBS 	+= -lrt
LIBS 	+= -lpthread
LIBS 	+= -lutil

comma = ,

# The dependency file for the current target
depfile = $(subst $(comma),_,$(dir $@).$(notdir $@).d)

DEPS	:= $(foreach obj,$(OBJS),\
		$(subst $(comma),_,$(dir $(obj)).$(notdir $(obj)).d))

DEFINES	+= -D_FILE_OFFSET_BITS=64
DEFINES	+= -D_GNU_SOURCE
DEFINES	+= -DBUILD_ARCH=riscv

KVM_INCLUDE := include
CFLAGS	+= $(CPPFLAGS) $(DEFINES) -I$(KVM_INCLUDE) -I$(ARCH_INCLUDE) -O2 -fno-strict-aliasing -g

WARNINGS += -Wall
WARNINGS += -Wformat=2
WARNINGS += -Winit-self
WARNINGS += -Wmissing-declarations
WARNINGS += -Wmissing-prototypes
WARNINGS += -Wnested-externs
WARNINGS += -Wno-system-headers
WARNINGS += -Wold-style-definition
WARNINGS += -Wredundant-decls
WARNINGS += -Wsign-compare
WARNINGS += -Wstrict-prototypes
WARNINGS += -Wundef
WARNINGS += -Wvolatile-register-var
WARNINGS += -Wwrite-strings
WARNINGS += -Wno-format-nonliteral

CFLAGS	+= $(WARNINGS)

ifneq ($(WERROR),0)
	CFLAGS += -Werror
endif

all: $(PROGRAM) static-lib

# CFLAGS used when building objects
# This is intentionally not assigned using :=
c_flags	= -Wp,-MD,$(depfile) -Wp,-MT,$@ $(CFLAGS)

# When building -static all objects are built with appropriate flags, which
# may differ between static & dynamic .o.  The objects are separated into
# .o and .static.o.  See the %.o: %.c rules below.
#
# $(OTHEROBJS) are things that do not get substituted like this.
#
STATIC_OBJS = $(patsubst %.o,%.static.o,$(OBJS) $(OBJS_STATOPT))

STATIC_DEPS	:= $(foreach obj,$(STATIC_OBJS),\
		$(subst $(comma),_,$(dir $(obj)).$(notdir $(obj)).d))

$(PROGRAM)-static:  $(STATIC_OBJS) $(OTHEROBJS) $(GUEST_OBJS)
	$(E) "  LINK    " $@
	$(Q) $(CC) -static $(CFLAGS) $(STATIC_OBJS) $(OTHEROBJS) $(GUEST_OBJS) $(LDFLAGS) $(LIBS) $(LIBS_STATOPT) -o $@

$(PROGRAM): $(OBJS) $(OBJS_DYNOPT) $(OTHEROBJS) $(GUEST_OBJS)
	$(E) "  LINK    " $@
	$(Q) $(CC) -c $(CFLAGS) $(OBJS) $(OBJS_DYNOPT) $(OTHEROBJS) $(GUEST_OBJS) $(LDFLAGS) $(LIBS) $(LIBS_DYNOPT) -o $@

$(OBJS):

util/rbtree.static.o util/rbtree.o: util/rbtree.c
ifeq ($(C),1)
	$(E) "  CHECK   " $@
	$(Q) $(CHECK) -c $(CFLAGS) $< -o $@
endif
	$(E) "  CC      " $@
	$(Q) $(CC) -c $(c_flags) $< -o $@

%.static.o: %.c
ifeq ($(C),1)
	$(E) "  CHECK   " $@
	$(Q) $(CHECK) -c $(CFLAGS) $(CFLAGS_STATOPT) $< -o $@
endif
	$(E) "  CC      " $@
	$(Q) $(CC) -c $(c_flags) $(CFLAGS_STATOPT)  $< -o $@

%.o: %.c
ifeq ($(C),1)
	$(E) "  CHECK   " $@
	$(Q) $(CHECK) -c $(CFLAGS) $(CFLAGS_DYNOPT) $< -o $@
endif
	$(E) "  CC      " $@
	$(Q) $(CC) -c $(c_flags) $(CFLAGS_DYNOPT) $< -o $@

static-lib:
	ar rcs liblkvm.a net.o blk.o mmio.o core.o disk/*.o util/*.o

clean:
	$(E) "  CLEAN"
	$(Q) rm -f $(DEPS) $(STATIC_DEPS) $(OBJS) $(OTHEROBJS) $(OBJS_DYNOPT) $(STATIC_OBJS) $(PROGRAM) $(PROGRAM_ALIAS) $(PROGRAM)-static $(GUEST_INIT) $(GUEST_PRE_INIT) $(GUEST_OBJS) liblkvm.a
.PHONY: clean
