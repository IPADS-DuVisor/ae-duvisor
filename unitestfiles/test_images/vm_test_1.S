/*
 * Copyright (c) 2020 Institute of Parallel And Distributed Systems (IPADS), Shanghai Jiao Tong University (SJTU)
 * ChCore is licensed under the Mulan PSL v1.
 * You can use this software according to the terms and conditions of the Mulan PSL v1.
 * You may obtain a copy of Mulan PSL v1 at:
 *   http://license.coscl.org.cn/MulanPSL
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR
 *   IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR
 *   PURPOSE.
 *   See the Mulan PSL v1 for more details.
 */

#include "asm.h"

BEGIN_FUNC(vm_test_1)
	li	a0,	0
	li  a0, 0
	li  a0, 0
	li	a1,	1
	li	a2,	2
	mv	t0, a0
	mv	t1,	a1
	mv	t2,	a2
	li	t4, 0x2000
	//csrr t5, misa
	//csrr t5, 0x64a
	//csrr t5, 0x600
	//csrr t5, 0x200
	//csrr t5, 0x200
	//csrr t5, 0x200
	ld	t2,	0(t4)
	li	t4, 0x3000
	ld	t3, 0(t4)
	add	t0,	t0,	t1
	mul t2, t2, t0
	ecall

.global vm_test_1_end;
vm_test_1_end:
END_FUNC(vm_test_1)
