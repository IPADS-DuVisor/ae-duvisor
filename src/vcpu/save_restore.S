// save host ctx
.macro SAVE_HOST_CTX reg1
	// save host gp with reg1=ctx, except t0-t6 and zero-x0
	SAVE_HOST_GP_X1 \reg1, x1
	SAVE_HOST_GP_X2 \reg1, x2
	SAVE_HOST_GP_X3 \reg1, x3
	SAVE_HOST_GP_X4 \reg1, x4
	SAVE_HOST_GP_X8 \reg1, x8
	SAVE_HOST_GP_X9 \reg1, x9
	SAVE_HOST_GP_X10 \reg1, x10
	SAVE_HOST_GP_X11 \reg1, x11
	SAVE_HOST_GP_X12 \reg1, x12
	SAVE_HOST_GP_X13 \reg1, x13
	SAVE_HOST_GP_X14 \reg1, x14
	SAVE_HOST_GP_X15 \reg1, x15
	SAVE_HOST_GP_X16 \reg1, x16
	SAVE_HOST_GP_X17 \reg1, x17
	SAVE_HOST_GP_X18 \reg1, x18
	SAVE_HOST_GP_X19 \reg1, x19
	SAVE_HOST_GP_X20 \reg1, x20
	SAVE_HOST_GP_X21 \reg1, x21
	SAVE_HOST_GP_X22 \reg1, x22
	SAVE_HOST_GP_X23 \reg1, x23
	SAVE_HOST_GP_X24 \reg1, x24
	SAVE_HOST_GP_X25 \reg1, x25
	SAVE_HOST_GP_X26 \reg1, x26
	SAVE_HOST_GP_X27 \reg1, x27
.endm

// restore host ctx, reg1 must be A0(X10)
.macro RESTORE_HOST_CTX reg1
	/* restore host gp with a0=ctx, except t0-t6 and zero-x0 */
    RESTORE_HOST_GP_X1 \reg1, x1
    RESTORE_HOST_GP_X2 \reg1, x2
    RESTORE_HOST_GP_X3 \reg1, x3
    RESTORE_HOST_GP_X4 \reg1, x4
    RESTORE_HOST_GP_X5 \reg1, x5
    RESTORE_HOST_GP_X6 \reg1, x6
    RESTORE_HOST_GP_X7 \reg1, x7
    RESTORE_HOST_GP_X8 \reg1, x8
    RESTORE_HOST_GP_X9 \reg1, x9
    RESTORE_HOST_GP_X10 \reg1, x10
    RESTORE_HOST_GP_X11 \reg1, x11
    RESTORE_HOST_GP_X12 \reg1, x12
    RESTORE_HOST_GP_X13 \reg1, x13
    RESTORE_HOST_GP_X14 \reg1, x14
    RESTORE_HOST_GP_X15 \reg1, x15
    RESTORE_HOST_GP_X16 \reg1, x16
    RESTORE_HOST_GP_X17 \reg1, x17
    RESTORE_HOST_GP_X18 \reg1, x18
    RESTORE_HOST_GP_X19 \reg1, x19
    RESTORE_HOST_GP_X20 \reg1, x20
    RESTORE_HOST_GP_X21 \reg1, x21
    RESTORE_HOST_GP_X22 \reg1, x22
    RESTORE_HOST_GP_X23 \reg1, x23
    RESTORE_HOST_GP_X24 \reg1, x24
    RESTORE_HOST_GP_X25 \reg1, x25
    RESTORE_HOST_GP_X26 \reg1, x26
    RESTORE_HOST_GP_X27 \reg1, x27
.endm

// save guest ctx, reg1 must be A0(X10)
.macro SAVE_GUEST_CTX reg1
	/* save guest gp except A0 & X0 */
    SAVE_GUEST_GP_X1 \reg1, x1
    SAVE_GUEST_GP_X2 \reg1, x2
    SAVE_GUEST_GP_X3 \reg1, x3
    SAVE_GUEST_GP_X4 \reg1, x4
    SAVE_GUEST_GP_X5 \reg1, x5
    SAVE_GUEST_GP_X6 \reg1, x6
    SAVE_GUEST_GP_X7 \reg1, x7
    SAVE_GUEST_GP_X8 \reg1, x8
    SAVE_GUEST_GP_X9 \reg1, x9
    SAVE_GUEST_GP_X11 \reg1, x11
    SAVE_GUEST_GP_X12 \reg1, x12
    SAVE_GUEST_GP_X13 \reg1, x13
    SAVE_GUEST_GP_X14 \reg1, x14
    SAVE_GUEST_GP_X15 \reg1, x15
    SAVE_GUEST_GP_X16 \reg1, x16
    SAVE_GUEST_GP_X17 \reg1, x17
    SAVE_GUEST_GP_X18 \reg1, x18
    SAVE_GUEST_GP_X19 \reg1, x19
    SAVE_GUEST_GP_X20 \reg1, x20
    SAVE_GUEST_GP_X21 \reg1, x21
    SAVE_GUEST_GP_X22 \reg1, x22
    SAVE_GUEST_GP_X23 \reg1, x23
    SAVE_GUEST_GP_X24 \reg1, x24
    SAVE_GUEST_GP_X25 \reg1, x25
    SAVE_GUEST_GP_X26 \reg1, x26
    SAVE_GUEST_GP_X27 \reg1, x27
    SAVE_GUEST_GP_X28 \reg1, x28
    SAVE_GUEST_GP_X29 \reg1, x29
    SAVE_GUEST_GP_X30 \reg1, x30
    SAVE_GUEST_GP_X31 \reg1, x31
.endm

// restore guest ctx, reg1 must be A0(X10)
.macro RESTORE_GUEST_CTX reg1
	// restore guest GP except A0 & X0
    RESTORE_GUEST_GP_X1 \reg1, x1
    RESTORE_GUEST_GP_X2 \reg1, x2
    RESTORE_GUEST_GP_X3 \reg1, x3
    RESTORE_GUEST_GP_X4 \reg1, x4
    RESTORE_GUEST_GP_X5 \reg1, x5
    RESTORE_GUEST_GP_X6 \reg1, x6
    RESTORE_GUEST_GP_X7 \reg1, x7
    RESTORE_GUEST_GP_X8 \reg1, x8
    RESTORE_GUEST_GP_X9 \reg1, x9
    RESTORE_GUEST_GP_X11 \reg1, x11
    RESTORE_GUEST_GP_X12 \reg1, x12
    RESTORE_GUEST_GP_X13 \reg1, x13
    RESTORE_GUEST_GP_X14 \reg1, x14
    RESTORE_GUEST_GP_X15 \reg1, x15
    RESTORE_GUEST_GP_X16 \reg1, x16
    RESTORE_GUEST_GP_X17 \reg1, x17
    RESTORE_GUEST_GP_X18 \reg1, x18
    RESTORE_GUEST_GP_X19 \reg1, x19
    RESTORE_GUEST_GP_X20 \reg1, x20
    RESTORE_GUEST_GP_X21 \reg1, x21
    RESTORE_GUEST_GP_X22 \reg1, x22
    RESTORE_GUEST_GP_X23 \reg1, x23
    RESTORE_GUEST_GP_X24 \reg1, x24
    RESTORE_GUEST_GP_X25 \reg1, x25
    RESTORE_GUEST_GP_X26 \reg1, x26
    RESTORE_GUEST_GP_X27 \reg1, x27
    RESTORE_GUEST_GP_X28 \reg1, x28
    RESTORE_GUEST_GP_X29 \reg1, x29
    RESTORE_GUEST_GP_X30 \reg1, x30
    RESTORE_GUEST_GP_X31 \reg1, x31
    /* restore guest reg1 */
    RESTORE_GUEST_GP_X10 \reg1, x10
.endm